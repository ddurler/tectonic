# Tectonic (r)

.__Extrait du journal 'Sport Cérébral__

`Tectonic` est un jeu de logique.

Il faut compléter une grille avec les chiffres manquants dans chaque zone entourés de gras, sachant que :

1. Une zone de deux cases contient les chiffres 1 et 2, une zone de 3 cases les chiffres 1, 2 et 3, etc.
2. Un chiffre placé dans une case ne peut se retrouver dans aucune des cases qui l'entoure (en diagonale y compris).

**Pour commencer :** Repérez, s'il y en a, les zones à une case : Elles contiennent uniquement le chiffre 1.

.__fin de l'extrait__

Représenter des zones entourées de gras étant incommode dans une fichier plat, les zones sont marquées par une lettre.

Exemple :

```txt
a1 b  b2
b4 b  b
c  c  c2
```

Cette définition représente une grille contenant 3 zones :

* Une zone notée 'a' c'une seule case en haut à gauche avec le chiffre 1 placé.
* Une zone notée 'b' qui contient déjà les chiffres 2 et 4 placés.
* Une zone notée 'c' sur la ligne du bas qui contient le chiffre 2 placé.

La solution de cette grille étant :

```txt
a1 b3 b2
b4 b5 b1
c1 c3 c2
```
