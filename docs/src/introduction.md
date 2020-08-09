# Introduction

Traditionnellement les marchés sont organisés en double enchères continues, ce système permet la
mise en relation des ordres aux prix désirés par les participants. Cependant ce systeme est sensible
au problème de double coincidence des besoins notamment sur les marchés peu liquides, une manière de
contourner ce problème est de passer par un intermédiaire.

Ce dernier peu mettre en relation les ordres de ses differents clients ou trouver des contre-parties
comme c'est le cas avec les courtiers. Les intermédiaires peuvent également se porter eux même comme
contre-partie et ainsi prendre les ordres, gérant ainsi leur propre portefeuille créant ainsi de la
liquidité sur le marché. C'est actuellement le cas sur le _NASDAQ_ ou le _NYSE_ avec des
spécialistes qui gèrent leur portefeuille et proposent des prix plus justes que les agents non
spécialisés. Cela permet, sur les marchés peu liquides, de réduire le tatonnement vers le prix
d'équilibre et ainsi augmenter le nombres d'échanges.

Cependant il existe certains marchés, appelés "marchés fins" (_thin markets_), ou le nombre
d'acheteurs et de vendeurs est tellement faible qu'aucun des acteurs ne pense trouver une
contrepartie et donc aucun n'entre sur le marché. C'est notamment le cas pour les marchés de
l'information qui permettent des ordres composés de plusieurs actifs simultanément, complexifiant la
mise en relation des ordres. Une solution à ce problème est d'utiliser un animateur de marché
automatisé, il s'agit d'un intermediaire contrepartiste toujours disponible à l'échange rendant le
marché entièrement liquide.

Ce projet présente une implémentation d'un animateur de marché automatisé. Dans une première partie
nous nous interesserons aux fondements théoriques des animateurs de marchés et à la programmation
asynchrone. Tandis que dans un second temps nous présenterons l'architecture, l'implémentation et
les résultats de l'application.
