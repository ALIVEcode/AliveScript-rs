# Structures conditionnelles

## Si

AliveScript supporte les structures conditionnelles `si`.

Exemple:

```as
lire entier dans var x, "Quel est ton âge?"

si x < 18 alors
    afficher "Tu es mineur"
sinon si x <= 30
    afficher "Tu es majeur"
sinon
    afficher "Tu as plus de 30 ans!"
fin si
```

Syntaxe:

- si: <code>si _condition_ [alors]</code>
- sinon si: <code>sinon si _condition_ [alors]</code>
- sinon: `sinon`
- fermeture: `fin si`
