Membres du groupe
=================

- Nicolas BONNEAU
- Rémi NICOLE

Rendu
=====

Ce qui est fait
---------------

- Makefile
	- make
	- make check
	- make run (run en mode release)
	- make doc (génère de la doc html dans target/doc/inf_4301a/index.html)
	- Propose l'installation de Rust
- Arithmétique et comparaisons
- Print
	- print (sans retour à la ligne)
	- println (avec retour à la ligne)
- If Then Else
- Scopes
	- Variables
	- Affectations
	- Fonctions (non first-class)
- Boucle while
- Boucle for
- Strings
- Type checker
- Conversions entre types
- Tableaux
- Tuples
- Pattern matching
- Types génériques (mais pas de support dans la syntaxe)

Difficultés rencontrées
-----------------------

- Implémenter (niveau design) les types génériques à l'évaluation (les
  types génériques à la compilation semblent plus simples)

Originalités
------------

- En Rust
- Un joli REPL avec le support du multiligne et de la complétion
- Des jolies erreurs qui pointent sur le code
- Un fuzzer est installé: pour l'exécuter, faites simplement
  `cargo fuzz run --release fuzzer_script_1`

Limitations
-----------

- Le type checker (qui produit toutes les erreurs) s'arrête à la première
  erreur (ce qui peut être frustrant pour un développeur dans un vrai langage
  de programmation)
- La complétion n'est pas parfaite (requiert que le le code partiel ne soit
  pas valide en terme de parsing)
- Le programme ne gère pas les erreurs à l'évaluation (en dehors du type
  checker), ce qui fait que les erreurs de type division par zéro, overflow,
  etc. font planter directement le programme, et que par exemple parser une
  string pour la convertir en Integer, Bool, etc. n'est pour l'instant pas
  possible.
