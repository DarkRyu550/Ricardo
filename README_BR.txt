#########################################
# Repositorio dos exercicios do Ricardo #
#                                       #
# Matheus Branco Borella (11218897)     #
#########################################

Este repositorio contem todos os meus projetos que foram entregues como exerci-
cios da materia de Computacao Grafica do primeiro semestre de 2021, juntamente
com suas bibliotecas de suporte. Todos os projetos desenvolvidos nesse reposito-
rio foram escritos na linguagem de programacao Rust.

# Estrutura do Projeto
Este projeto eh estruturado em tres pastas principais, uma das quais eh composta
de subpastas que tambem sao projetos, essas sao:
    - "environment/": Uma pequena biblioteca que evita o boilerplate que seria
      normalmente necessario para criar uma janela de aplicativo e calcular
      tempos tanto na plataforma normal como na plataforma web.
    - "gavle/": Uma biblioteca, de minha autoria, que emula conceitos nativos a
      APIs como Vulkan e Metal sobre o OpenGL ES 3. Alem disso, ela tambem in-
      troduzir estruturas de controle implicitas para acesso simultaneo a recur-
      sos como attachments de cor e profundidade-stencil durante a execucao de
      pipelines, de tal forma que tem-se a todo momento garantia de que um dado
      recurso soh pode estar em um de dois estados: Ou com acesso compartilhado
      em modo somente leitura, ou com acesso exclusivo em modo leitura-escrita,
      o que evita comportamentos nao definidos por parte da API executando os
      comandos de renderizacao.
    - "exercises/": A pasta que contem os subprojetos de cada um dos exercicios,
      nomeados pelo seu numero escrito por extenso em ingles. A pasta
      "excercises/support" eh uma excessao a regra de nomes por conter codigo de
      suporte compartilhado entre todos os exercicios.

# Compilacao e Execucao
Para compilar esse projeto, eh necessario que se possua tanto a versao mais re-
cente do compilador `rustc` assim como da ferramenta padrao de gerenciamento de
pacotes da linguagem, chamada `cargo`.

Ambas podem ser instaladas por meio do executavel obtido em https://rustup.rs,
que toma conta de manuzear diversas insalacoes das ferramentas da linguagem para
um dado usuario.

Um dado exercicio eh compilado executado usando a seguinte forma de linha de co-
mando:
```
$ cargo run --release -p exercise-<numero>
```

Onde `<numero>` eh um numero cardinal escrito por extenso, em ingles: "one",
"two", "three", etc.

# Exercicios Atualmente Implementados
Os seguintes exercicios jah foram implementados:
    - [x] Exercicio #1 (`exercise-one`): Programa que utiliza as primitivas 2D para
          gerar um desenho abstrato.
    - [x] Exercicio #2-A (`exercise-two-a`): Programa que desenha um triangulo e
          aplica uma transformacao geometrica de escala uniforme.
    - [x] Exercicio #2-B (`exercise-two-b`): Programa que desenha um retangulo
          utilizando o modo de TriangleStrip e em seguida aplica uma rotacao
          que pode ser controlada com as setas do teclado.
    - [x] Exercicio #2-C (`exercise-two-c`): Programa que desenha um circulo
          cujo centro pode ser translado para uma posicao controlada usando as
          teclas WASD do teclado.
    - [x] Exercicio #2-D (`exercise-two-d`): Programa que desenha um triangulo cuja
          posicao segue a posicao do mouse e um circulo que funciona igual ao
          implementado no exercicio #2-C.
    - [x] Exercicio #2-E (`exercise-two-e`): Programa que desenha um modelo
          de um disco que pode ser movimentado usando o mouse e arrastando.
    - [x] Exercicio #3-A (`exercise-three-a`): Programa que desenha um modelo
          de uma esfera que pode ser movimentada usando o mouse e arrastando.
    - [x] Exercicio #2-E (`exercise-three-b`): Programa que desenha um modelo
          de um cilindro que pode ser movimentado usando o mouse e arrastando.