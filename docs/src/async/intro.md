# Programmation Asynchrone

La programmation asynchrone opposée à la programmation linéaire, désigne l'indépendance de l'ordre
des événements dans l'éxecution du programme.

La nature de ces événements est généralement lié à l'attente de la fin d'une action d'un autre
processus (pour les temps de calcul longs) ou d'un autre ordinateur lors d'une connection. Cette
approche permet, notamment, de gerer une multitude de connection sans lancer un processus
indépendant. En effet, que toutes les connections ne sont pas actives simultanément. Par exemple,
pour construire un marché tous les clients connectés n'emettent pas des ordres continuellement,
certains sont plus actifs que d'autres. Plutot que d'assumer des fréquences d'ordre et verifier par
order de priorité les connections, on utilise de la programmation asynchrone afin que dès que de
l'information peut être récupéré les taches associées soient executés.

Dans le cadre du projet nous allons voir le modèle de programmation asynchrone en language
[**Rust**](https://www.rust-lang.org/), les prérequis afin d'éxécuter les objets générés par le
language et enfin lier cela aux ressources Input/Output (I/O) sur un système linux.
