# Programmation Asynchrone

La programmation asynchrone opposée à la programmation linéaire, désigne l'indépendance de l'ordre
des événements dans l'éxecution du programme.

La nature de ces événements est généralement lié à l'attente de la fin d'une action d'un autre
processus (pour les temps de calcul longs) ou d'un autre ordinateur si les deux sont connectés.
Cette approche permet, notamment, de gérer une multitude de connections sans lancer de processus
indépendants. En effet, toutes les connections ne sont pas actives simultanément. Par exemple, pour
construire un marché tous les clients connectés n'emettent pas des ordres continuellement, certains
sont plus actifs que d'autres. Plutot que d'assumer des fréquences d'ordre et vérifier par order de
priorité les connections, on utilise de la programmation asynchrone afin que dès que de
l'information peut être récupéré les taches associées soient executés.

Dans le cadre du projet nous allons voir le modèle de programmation asynchrone en language
[**Rust**](https://www.rust-lang.org/).
