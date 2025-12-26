use std::cell::RefCell;
use std::rc::Rc;

use pyo3::types::{PyDict, PyList};
use pyo3::{prelude::*, types::PyFunction};

use crate::as_obj::{ASDict, ASFnParam, ASObj, ASScope, ASType, ASVar};
use crate::runner::Runner;

pub fn run_python_script(script: String) -> Option<Rc<RefCell<ASScope>>> {
    let mut env = ASScope::new();
    let result: PyResult<()> = Python::with_gil(|py| {
        let locals = PyDict::new(py);
        py.run(&script, None, Some(&locals))?;
        for (var, val) in locals.iter() {
            let var = ASVar::new(var.to_string(), None, false);
            let val = py_obj_to_as_obj(py, &val);
            env.declare(var, val);
        }
        Ok(())
    });

    if result.is_err() {
        return None;
    }

    Some(Rc::new(RefCell::new(env)))
}

fn py_obj_to_as_obj(py: Python<'_>, py_obj: &PyAny) -> ASObj {
    if py_obj.is_none() {
        return ASObj::ASNul;
    }
    if let Ok(val) = py_obj.extract::<String>() {
        return ASObj::ASTexte(val);
    }
    if let Ok(val) = py_obj.extract::<i64>() {
        return ASObj::ASEntier(val);
    }
    if let Ok(val) = py_obj.extract::<f64>() {
        return ASObj::ASDecimal(val);
    }
    if let Ok(val) = py_obj.extract::<bool>() {
        return ASObj::ASBooleen(val);
    }
    if let Ok(vals) = py_obj.extract::<Vec<Py<PyAny>>>() {
        let mut list = Vec::new();
        for val in vals {
            list.push(py_obj_to_as_obj(py, val.downcast::<PyAny>(py).unwrap()));
        }
        return ASObj::ASListe(Rc::new(RefCell::new(list)));
    }
    if let Ok(d) = py_obj.extract::<Py<PyDict>>() {
        let mut dict = ASDict::default();
        for (key, val) in d.as_ref(py).iter() {
            let key = py_obj_to_as_obj(py, key);
            let val = py_obj_to_as_obj(py, val);
            dict.insert(key, val);
        }
        return ASObj::ASDict(Rc::new(RefCell::new(dict)));
    }
    if let Ok(f) = py_obj.extract::<Py<PyFunction>>() {
        let f = f.clone();
        let name = f
            .as_ref(py)
            .getattr("__name__")
            .unwrap()
            .extract::<String>()
            .unwrap();
        let nparams = f
            .as_ref(py)
            .getattr("__code__")
            .unwrap()
            .getattr("co_argcount")
            .unwrap()
            .extract::<usize>()
            .unwrap();
        let params = f
            .as_ref(py)
            .getattr("__code__")
            .unwrap()
            .getattr("co_varnames")
            .unwrap()
            .extract::<Vec<String>>()
            .unwrap()
            .into_iter()
            .take(nparams)
            .map(|param| ASFnParam::new(param, None, None))
            .collect::<Vec<ASFnParam>>();
        return ASObj::native_fn(
            name.as_str(),
            None,
            params.clone(),
            Rc::new(move |runner: &mut Runner| {
                let result = Python::with_gil(|py| {
                    let args = PyList::empty(py);
                    for param in params.iter() {
                        args.append(as_obj_to_py_obj(
                            py,
                            &runner.get_env().get_value(&param.name).unwrap(),
                        ))
                        .unwrap();
                    }
                    py_obj_to_as_obj(py, &f.as_ref(py).call(args.to_tuple(), None).unwrap())
                });
                Ok(Some(result))
            }),
            ASType::any(),
        );
    }
    if let Ok(module) = py_obj.extract::<Py<PyModule>>() {
        return ASObj::ASNul;
    }
    todo!("finir py_obj_to_as_obj {}", py_obj)
}

fn as_obj_to_py_obj(py: Python<'_>, as_obj: &ASObj) -> PyObject {
    match as_obj {
        ASObj::ASTexte(val) => val.to_object(py),
        ASObj::ASEntier(val) => val.to_object(py),
        ASObj::ASDecimal(val) => val.to_object(py),
        ASObj::ASBooleen(val) => val.to_object(py),
        ASObj::ASListe(val) => {
            let mut list = Vec::new();
            for val in val.borrow().iter() {
                list.push(as_obj_to_py_obj(py, &val));
            }
            list.to_object(py)
        }
        ASObj::ASDict(val) => {
            let dict = PyDict::new(py);
            for item in val.borrow().items() {
                dict.set_item(
                    as_obj_to_py_obj(py, item.key()),
                    as_obj_to_py_obj(py, item.val()),
                )
                .unwrap();
            }
            dict.to_object(py)
        }
        ASObj::ASNul => py.None(),
        _ => todo!("Finir as_obj_to_py_obj"),
    }
}
