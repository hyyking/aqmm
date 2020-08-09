# Protocol

La communication entre le serveur et les client se fait par message rpc encodé en protobuf, dans
cette partie nous verrons les détails.

## Remote Procedure Call (RPC)

Le RPC est un protocole réseau permettant l'éxécution de taches sur un serveur depuis un client.
Cette solution semble être bonne pour construire un marché ou les ordres sont demandé par le client
et effectué par le serveur.

Le client envois un premier une requête et bloque en attendant une réponse. Dans le cas de ce
serveur le client à le choix entre 3 requêtes:

1. Requête d'identification: permet au serveur de charger la session ou d'en créer une si elle
   n'existe pas, la réponse contient l'identifiant de la session.
2. Requête de la liste des titres: le serveur renvoie la liste des titres disponible avec une courte
   description de chacuns.
3. Requête d'ordres: le serveur execute et renvois le résultat des ordres.

## Protobuf

Protocol Buffers est un format de sérialisation et de description d'interface développé par Google.
Originellement conçut pour un nombre limité de languages nous utiliserons une implementation
open-source pour Rust appelé [prost!](https://docs.rs/prost/0.6.1/prost/).

### Language de description d'interface

Les language de description d'interface permettent de définir des composants de logiciel dans un
language neutre de l'implementation de ce dernier. C'est à dire que si un fichier de description est
partagé entre un serveur et un client, ils pourront être implémenté dans différents languages de
programmation car ils auront la description des éléments pour désérialiser les données.

Ex:

Le protobuf

```protobuf
message Ordre {
	uint64 qt = 1;
	double prix = 2;
}
```

devient en rust:

```rust, ignore
pub struct Ordre {
	qt: u64,
	prix: f64,
}
```

et en java:

```java, ignore
public class Ordre {
  long qt;
  double prix;
}
```

### Sérialisation/Désérialisation

La désérialisation est le fait de réduire une structure de donnée d'un programme en un ensemble
cohérent en binaire. La sérialisation, qui est l'inverse de la désérialisation, est le fait de créer
un structure de donnée depuis un ensemble cohérent d'information binaire.

Couplé ensemble ils permettent de stocker des structures de données dans des bases de données ou
d'envoyer ces premières à travers des réseaux. En effet, si un client et un serveur possèdent la
même structure ils peuvent s'échanger des information structuré facilement en restructurant les
informations de l'autre.

## Transmission

Les clients se connectent via TCP (Transmission Control Protocol) pour emmettre les message RPC.

Concernant les information de quantité sont relayé via un système de multicasting en UDP (User
Datagram Protocol), c'est à dire un système ou le serveur envoie de la donnée à une adresse et les
clients qui le souhaite récupèrent la donnée sans que le serveur ai à entretenir une connection ou
compter les personnes lisant les quantités. Auquel cas Le message protobuf envoyé est `Broadcast`.

## Protocol entier

```proto
{{#include ../../../proto/aqmm.proto}}
```
