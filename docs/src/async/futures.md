# Future et Stream en language Rust

Le language Rust n'est pas un language orienté objet comme les autres, en effet il n'existe pas
d'objets à proprement parler. Cependant, les structures peuvent avoir des methodes associés uniques
et d'autres apportés par un _trait_ (penser trait de charactère `content -> sourire()`). Ainsi pour
définir les calculs asynchrones le language fournit le _trait_ `std::future::Future` et pour les
itérateurs asynchrones `futures::stream::Stream` qui se trouve dans la librarie
[futures](https://docs.rs/futures/0.3.5/futures/index.html) maintenu par les developpeurs du
language en attendant d'être stabilisé.

Dans cette partie nous verrons comment est définit le _trait_ `std::future::Future`, en quoi il
définit un calcul asynchrone et comment il est utilisé en pratique. Enfin nous étendrons ça aux
itérateurs asynchrones avec le _trait_ `futures::stream::Stream`.

## `std::future::Future`

### Définition

Le _trait_ est définit de la façon suivante.

```rust, ignore
pub trait Future {
    type Output;
    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output>;
}
```

Lorsque nous décomposons nous voyons qu'il y a un type associé `type Output;` qui correspond au type
de l'élément retourné par le future. Concernant la méthode, elle retourne `Poll<T>` un _enum_, avec
un paramètre générique `T`, pouvant prendre deux formes:

```rust, ignore
pub enum Poll<T> {
	Ready(T),
	Pending
}
```

Ainsi, lorsque que la fonction retourne `Poll::Ready(10)` on dit que le future est terminé car il a
produit une valeur. Tandis que lorsque `Poll::Pending` est renvoyé cela veut dire qu'il n'est pas
encore terminé et il doit être reveillé par un évènement qui indique qu'il va peut-être renvoyer
`Poll::Ready(_)`. Pour que le future soit reveillé il reçoit une référence (`&mut`) à un `Context`,
dont il peut recuperer le `Waker`, structure qui permet de reveiller la tache associé, et le stocke
quelque part pour qu'un autre processus puisse le reveiller.

PS: Nous ne nous attarderons pas sur `Pin<&mut Self>` qui assure seulement que le future n'est pas
remplacé pendant un appel à `poll`.

### Calcul

En général les futures utilisés en rust sont issues de la combinaison de plusieurs futures. Avant
novembre 2019 et la sortie de la version 1.39.0 les combinaison étaient effectués avec des
structures dites combinatoires.

#### Combinatoire pre 1.39

##### Composition sequentielle:

- _Ex_: `f.and_then(|output_future_precedent| nouveau_future(output_future_precedent))`
- _Implication_: lorsque future `f` est executé jusqu'au bout, un nouveau future est construit du
  resultat du précédent

##### Changement de type:

- _Ex_: `f.map(|output_future_precedent| nouveau_type(output_future_precedent))`
- _Implication_: le type `Output` du future `f` est passé dans une fonction lui donnant un nouveau
  type (modifiant ainsi sa valeur à la fin de l'éxécution).

##### Jointure:

- _Ex_: `f.join(g)`
- _Implication_: les futures `f` et `g` sont éxécuté parallement et le nouveau future se termine
  lorsque les deux sont terminés.

##### Selection:

- _Ex_: `f.select(g)`
- _Implication_: les futures `f` et `g` sont executé parallement et le nouveau future se termine
  lorsqu'un des deux est terminé.

#### Mise à jour 1.39

Depuis la mise à jour 1.39.0 la syntaxe `async/await` à été stabilisé permettant d'utiliser les
futures comme du code classique. Pour recuperer le resultat d'un future il faut symplement `await`,
ce qui execute le future, dans une fonction `async`hrone, qui indique elle meme etre un future (nous
reviendrons sur la récupération du résultat d'un future dans la partie suivante).

Ex:

```rust, ignore
// Ici le type de retour est implicitement `impl Future<Output = String>`
// Soit une "structure qui est un future avec comme resultat un pointeur de texte"
async fn demo() -> String {
	// création et attente du resultat du future
	let future1: u64 = Future1::new().await;
	assert!(future1 == 10);

	// création du future sans attendre le resultat
	let future2 = Future2::new();

	// attente du résultat et changement de type + retour implicite de la fonction
	String::from(future2.await)
}
```

Cette syntaxe rend la combinatoire obselète car on travaille toujours avec les valeurs directement,
permettant ainsi de les combiner ou de modifier leur type de manière plus aisé.

## `futures::stream::Stream`

Si un future est l'équivalent d'une promesse de valeur dans le temps, un `Stream` est lui
l'équivalent d'une succession de valeur qui arrivent à la suite. Pour comprendre ce principe il
convient de d'abord regarder le principe d'un itérateur pour l'étendre aux stream et en présenter
les applications.

### Définition

Tout d'abord définisson les itérateurs, qui sont des structures permettant de traverser un
collection comme par exemple une liste. Les itérateurs produisent des valeurs consommés par une
boucle. Les valeurs peuvent venir d'une liste, d'un calcul (suite de fibonacci par exemple), ou bien
d'une autre operation comme l'attente d'un message d'un autre processus. La différence première avec
un Stream est que l'itérateur bloque le processus à chacune de ses valeurs, le stream à l'instare
des futures attend d'être reveillé s'il est capable de produire une valeur. S'en suit la définition
suivante.

```rust

pub trait Stream {
    type Item;
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>>;
}
```

Tout comme les futures il y a un resultat (`Item`), un `Poll` et une référence à un `Context`.
Cependant pour determiner la fin d'un stream le language utilise un autre _enum_ commun
l'`Option<T>` qui prend (comme `Poll<T>`) une variante avec un objet (`Option::Some(T)`) et une sans
(`Option::None`). Ainsi on peu faire une disjonction de cas:

- `Poll::Ready(Some(Item))` => la valeur est prête à être consommée.
- `Poll::Ready(None)` => le stream ne produira plus de valeurs.
- `Poll::Pending` => la prochaine valeur n'est pas encore prête un processus reveillera ce stream
  lorsque ce sera le cas.

### Calcul

Les streams possèdent également des combinatoires qui se trouve dans le _trait_
[`futures::stream::StreamExt`](https://docs.rs/futures/0.3.5/futures/stream/trait.StreamExt.html).
Les principales sont:

##### Next

- _Ex_: `stream.next()`
- _Implication_: Produit un future qui se résout lorsque la prochaine valeur arrive ou le stream est
  terminé.

##### Map

- _Ex_: `stream.map(|val| { calcul(val) })`
- _Implication_: Produit un stream ou tous les objets de type `Item` sont modifiés en un autre type

##### For Each

- _Ex_: `stream.for_each(|val| { calcul(val) })`
- _Implication_: Produit un future qui se résout lorsque toute les valeurs sont arrivées et ont été
  traitées.

##### Filter

- _Ex_: `stream.filter(|ref val| { val % 2 == 0 })`
- _Implication_: Produit un stream dont les éléments ou le calcul renvoit `false` ne sont pas
  retourné.
