use crate::ast::*;

pub trait Visitor {
    fn visit_body(&mut self, stmts: &Vec<Box<Stmt>>);

    fn visit_generic_expr(&mut self, expr: &Expr);
    fn visit_generic_stmt(&mut self, stmt: &Stmt);

    // Expressions
    fn visit_expr_lit(&mut self, expr: &Expr);
    fn visit_expr_list(&mut self, expr: &Expr);
    fn visit_expr_paire(&mut self, expr: &Expr);
    fn visit_expr_dict(&mut self, expr: &Expr);

    fn visit_expr_ident(&mut self, expr: &Expr);
    fn visit_expr_accessprop(&mut self, expr: &Expr);
    fn visit_expr_slice(&mut self, expr: &Expr);

    fn visit_expr_fncall(&mut self, expr: &Expr);
    fn visit_expr_classe_init(&mut self, expr: &Expr);
    fn visit_expr_callrust(&mut self, expr: &Expr);

    fn visit_expr_unaryop(&mut self, expr: &Expr);
    fn visit_expr_binop(&mut self, expr: &Expr);
    fn visit_expr_bincomp(&mut self, expr: &Expr);
    fn visit_expr_binlogic(&mut self, expr: &Expr);

    fn visit_expr_suite(&mut self, expr: &Expr);

    fn visit_expr_deffn(&mut self, expr: &Expr);
    fn visit_expr_faire(&mut self, expr: &Expr);

    // Statements
    fn visit_stmt_expr(&mut self, stmt: &Stmt);
    fn visit_stmt_afficher(&mut self, stmt: &Stmt);
    fn visit_stmt_lire(&mut self, stmt: &Stmt);

    fn visit_stmt_utiliser(&mut self, stmt: &Stmt);

    fn visit_stmt_decl(&mut self, stmt: &Stmt);
    fn visit_stmt_assign(&mut self, stmt: &Stmt);

    fn visit_stmt_opassign(&mut self, stmt: &Stmt);

    fn visit_stmt_si(&mut self, stmt: &Stmt);
    fn visit_stmt_condstmt(&mut self, stmt: &Stmt);

    fn visit_stmt_repeter(&mut self, stmt: &Stmt);
    fn visit_stmt_pour(&mut self, stmt: &Stmt);
    fn visit_stmt_tantque(&mut self, stmt: &Stmt);
    fn visit_stmt_continuer(&mut self, stmt: &Stmt);
    fn visit_stmt_sortir(&mut self, stmt: &Stmt);

    fn visit_stmt_retourner(&mut self, stmt: &Stmt);

    fn visit_stmt_deffn(&mut self, stmt: &Stmt);
    fn visit_stmt_defclasse(&mut self, stmt: &Stmt);

    // Types
    fn visit_type_lit(&mut self, t: &Type);
    fn visit_type_binop(&mut self, t: &Type);
    fn visit_type_opt(&mut self, t: &Type);
}

pub trait Visitable {
    fn accept<V: Visitor>(&self, visitor: &mut V);
}
