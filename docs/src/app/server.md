# Serveur

Dans cette partie nous verrons a construction du serveur, ce dernier est répartit en plusieurs
modules le premier étant celui des ressources Input/Output c'est à dire concernant les abstractions
liés aux notifications système. Un deuxième module connexe au premier fait la liaison entre les
entrée de notification systeme et les connections TCP permettant à ces dernières d'être entièrement
asynchrones. Un dernier est lié au marché avec la liste des titres et les modules d'éxécution
d'ordre.

Structure des éléments de l'application (dossiers et fichiers non pertinents omis):

```
.
├── build.rs			-> script de compilation, génère le code protobuf
├── Cargo.toml			-> fichier de spécification du projet
├── src
│   ├── lib.rs
│   ├── ...
│   ├── net
│   │   ├── mod.rs
│   │   ├── codec.rs
│   │   └── tcp.rs
│   ├── market
│   │   ├── securities.rs
│   │   ├── core.rs
│   │   ├── mod.rs
│   │   ├── pool.rs
│   │   └── router.rs
│   ├── io
│   │   ├── mod.rs
│   │   ├── registration.rs
│   │   └── driver
│   │       ├── context.rs
│   │       └── mod.rs
│   ├── server.rs
│   └── bin			-> binaires éxécutables
├── proto
│   └── aqmm.proto		-> protocol en protobuf
└── ...
```

## Exécuteur et I/O

La structure du serveur (dans le fichier `src/server.rs`) est composé d'un "driver" pour les
ressources I/O liés aux connections arrivant. Le but du serveur est d'écouter les connections
arrivantes et pour chaque connection d'éxécuter un client qui prend la forme d'un future. A chaque
client est associé une connection ainsi qu'une session. Afin de determiner quels clients ont une
action de prête le serveur utilise la librairie [`mio`](https://docs.rs/mio) qui permet d'associer
des ressources I/O et d'emettre des notifications sur les connections ayant du travail. Les
évènements sont ensuite transferé à des entrées contenant les `std::task::Waker` des streams de
message des clients ces messages sont ensuite décodés et une réponse est envoyé suviant le protocole
décrit dans la partie précédente.

## Réseau

Le module `src/net` contient tous les éléments nécéssaire au maintient et à l'utilisation des
connections.

### Stream Tcp Asynchrone

Le driver de ressources I/O permet de savoir quand une connection est en mesure d'être écrite/lu. De
cette manière le driver peut reveiller les `Waker` associés. Il manque donc deux _trait_ comme
`std::future::Future` mais pour les actions de lecture et d'ecriture d'un flux de donnée. Le
language expose les _traits_ synchrone `std::io::Read` et `std::io::Write`. La libraire `futures`
définit de la même facon `futures::io::AsyncRead` et `futures::io::AsyncWrite`. Ainsi avec le driver
I/O nous pouvons construire une abstraction par dessus un TcpStream synchrone qui implémente
`AsyncRead + AsyncWrite`

### Codec

Les codecs sont des structures construites sur les _traits_ `tokio_util::codec::Encoder` et
`tokio_util::codec::Decoder` trouvé dans la librairie [`tokio_util`](https://docs.rs/tokio_codec).
Il permettent de lier l'utilisation de zone mémoire tampon avec l'encodage et le décodage de
messages. Le module expose un codec client, qui encode des requêtes et décode des réponses ainsi
qu'un codec serveur qui encode les réponses et décode les requètes.

L'interet d'exposer ces codec est de pouvoir utiliser la structure `tokio_util::codec::Framed`
exposé par la même libraire. La structure permet de créer un stream et un "sink" depuis un codec
(`Encoder + Decoder`) et une structure pouvant être lu/écrite de manière asynchrone
(`AsyncRead + AsyncWrite`). Le stream renvois ainsi les objets décodés et le "sink" permet d'envoyer
des objets qui seront encodés puis envoyés sur la connection.

## Marché

Le marché est composé de plusieurs éléments. La structure principale qui sert de pointeur vers un
état partagé qui comprend une liste des coeurs du marché et un accès au routeur d'ordres. Tous les
coeurs ont également un pointeur vers un une structure partagé contenant le compte de chaque titres
ainsi qu'un accès au multicast.

Le but de cette structure est de servire d'interface partagé entre tous les clients pour envoyer des
ordres en contrepartie d'un future sur le résultat de cet ordre.

### Routeur et coeurs du marché

Le router d'ordre est un structure faisant part du marché son but est d'envoyer les ordres sur les
différents coeurs. Les stratégies de routage des ordres peuvent être plus ou moins complexe. En
effet, le but des coeurs est d'éxécuter des ordres cependant on pourrait se retrouver dans une
situation ou un coeur recoit tous les ordres tandis que les autres ne font rien gachant ainsi du
temps de processeur. Par exemple, on pourrait imaginer une stratégie ou les ordres sont triés et en
fonction des titres visés sont envoyé sur un coeurs différent. Dans le cadre de ce projet le router
est plutot basique et agit comme une roue distribuant tour à tour à chaque coeur. Les points de
sortis des router implémente égalment le _trait_ `futures::stream::Stream` de manière à ce que les
coeurs ne soient actifs que lorsque des ordres arrivent (et ainsi les notifiants).

Le coeur du marché est lancé sur un processus différent, il s'agit d'une structure asynchrone qui
attend les ordres du router. Une fois l'ordre reçu il tente attend de pouvoir avoir un accès
exclusifs (via un [Mutex](https://en.wikipedia.org/wiki/Mutual_exclusion)) aux quantités actuelles
de l'animateur de marché. Une fois les quantités acquises il calcule le score pour les anciennes et
le score pour les nouvelles après avoir éxécuté tous les ordres determinant le prix de la
transaction. Une fois le verrous des quantités levé il tente de partager les nouvelles quantités,
cette action échouera si les quantités ont été partagé trop récemment.

### Titres

Le fichier `src/market/securities.rs` permet d'éditer les titres avant la compilation et contient
également des fonctions y facilitant l'accès. Le fichier définit également un autre type
`Securities` dont la description est du texte statique car il est, pour le moment, impossible de
créer statiquement (comprendre lors de la compilation) un pointeur de texte `String` requis par
prost pour les entrées type string en protobuf.

```rust,ignore
{{ #include ../../../src/market/securities.rs:9:22 }}
```
