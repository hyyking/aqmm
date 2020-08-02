# Executeur

Cette partie est l'extention de la partie précedente sur les `std::future::Future`, nous y
expliquons le principe d'un executeur de future.

## Execution d'un future

Comme vu precedement pour executer un future il faut un `Waker` permettant de se reveiller la tache.
Cependant que veut dire reveiller la tache, dans le cadre des futures nous avons vu qu'il s'agit de
rappeler `Futre::poll`. Cette tache est associé au concept de runtime dans d'autre language, en
effet en python ou en go le language fait tourner un processus qui se charge d'executer les calculs
asynchrones, ce processus est appelé "runtime". Le language rust essayant de se maintenir à un bas
niveau d'abstraction ne fournit pas un runtime, ainsi il s'agit de la tache du programmeur d'en
utiliser/programmer un. Des exemples de runtimes sont [tokio](https://docs.rs/tokio) et
[async-std](https://docs.rs/async-std).

## Bloquer sur un future

Comme il n'y a pas de runtime en language rust il faut la capacité de bloquer sur un futur à
l'origine des autres. Pour se faire un peu utiliser les capacité du système d'opération pour bloquer
les processus et ainsi ne pas créer un boucle qui attend que le future se termine avec un context
finalement inutile. Un [exemple](https://stjepang.github.io/2020/01/25/build-your-own-block-on.html)
minimaliste d'une telle fonction est le suivant:

```rust
fn block_on<F: Future>(future: F) -> F::Output {
    pin_utils::pin_mut!(future); // pour avoir un Pin<&mut Self> pour le future

    let parker = crossbeam::Parker::new();
    let unparker = parker.unparker().clone();
    let waker = async_task::waker_fn(move || unparker.unpark());

    let cx = &mut Context::from_waker(&waker);
    loop {
        match future.as_mut().poll(cx) {
            Poll::Ready(output) => return output,
            Poll::Pending => parker.park(),
        }
    }
}
```

Cette exemple utilise deux librairies (`crossbeam` et `async-task`) mais montre bien le processus
d'execution. Tant que le future n'est pas terminé on appel `Future::poll`, s'il ne se termine pas on
parque le processus sur l'os et on attend d'etre reveillé par le waker qui va l'`unpark`.

## Execution asynchrone

Sur l'exemple si dessus nous avons trouvé un moyen d'attendre le resultat d'un future sans executer
la boucle constamment (en parquant le processus). Cependant parfois on souhaite qu'un future soit
executé sans se soucier du resultat et ce de manière asynchrone. La plupart des runtime ont une
commande pour `spawn` un future qui sera executé par un ensemble de processus, souvent en utilisant
un systeme de vol de taches pour maintenir la bonne répartition de ces dernières à travers les
differents processus.

En étendant sur l'exemple précedent:

```rust
// Liste des taches
static QUEUE: Vec<Task> = Vec::new();

// pour lancer une tache on l'ajoute à la file
fn spawn<F: Future>(future: F) {
	// A chaque reveil on ajoute le future à la file d'attente
    let (task, _) = async_task::spawn(future, |f| QUEUE.push(f), ());
	task.schedule()
}

fn block_on<F: Future>(future: F) -> F::Output {
	/* ... */
    loop {
		// On execute les taches reveillés ici
		for task in QUEUE {
			task.run()
		}
        match future.as_mut().poll(cx) {
			/* ... */
        }
    }
}

fn main() {
	block_on(async {
		// stream de connections
		for connection in Listener::new() {
			spawn(async {
				// imprimer le prochain message
				println!(connection.next().await)
			});
		}
	})
}
```

De cette manière plusieurs taches peuvent être executés en parallèle sans avoir un processus associé
pour chacune.
