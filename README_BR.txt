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

Um dado exemplo eh compilado executado usando a seguinte forma de linha de co-
mando:
```
$ cargo run --release -p example-<numero>
```

Onde `<numero>` eh um numero cardinal escrito por extenso, em ingles: "one",
"two", "three", etc.

# Exemplos Atualmente Implementados
Os seguintes exercicios jah foram implementados:
    - [x] Exemplo #1 (`example-one`): Programa que utiliza as primitivas 2D para
      gerar um desenho abstrato.