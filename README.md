# CrabTask 🦀

**CrabTask** é um aplicativo simples e rápido de lista de tarefas (To-Do list) para terminal (TUI) escrito em Rust.

## Recursos

- **Gerenciamento de Tarefas:** Adicione, conclua e exclua tarefas rapidamente.
- **Categorias (Tags):** Crie categorias personalizadas com diversas cores (Vermelho, Amarelo, Verde, Ciano, Azul, Magenta, Branco, Cinza) e associe-as às suas tarefas.
- **Armazenamento Persistente:** Suas tarefas e categorias são salvas automaticamente em um arquivo `tasks.json` local.
- **Estatísticas:** Acompanhe o progresso de conclusão das suas tarefas diretamente na interface.
- **Interface TUI:** Uma interface de usuário de terminal moderna e responsiva baseada no Ratatui.

## Como Executar

Certifique-se de ter o [Rust e o Cargo](https://rustup.rs/) instalados no seu sistema.

Para executar o projeto, basta rodar o seguinte comando na raiz do repositório:

```bash
cargo run --release
```

## Atalhos do Teclado (Keybindings)

### Modo de Lista (Normal)
- `a` : Adicionar uma nova tarefa (entra no modo de inserção).
- `d` : Excluir a tarefa selecionada.
- `Espaço` : Marcar/desmarcar a tarefa selecionada como concluída.
- `c` : Criar uma nova categoria.
- `↓` / `j` : Selecionar a próxima tarefa.
- `↑` / `k` : Selecionar a tarefa anterior.
- `1` a `9` : Adicionar/remover a tag (categoria correspondente a este número) da tarefa selecionada.
- `q` / `Esc` : Sair do aplicativo.

### Modo de Inserção (Nova Tarefa)
- `Enter` : Confirmar a criação da tarefa.
- `Esc` : Cancelar a inserção.

### Modo de Edição de Categoria
- `Tab` : Alternar foco entre o nome da categoria e a seleção de cor.
- `←` / `h` e `→` / `l` : Mudar a cor selecionada.
- `Enter` : Confirmar a criação da categoria.
- `Esc` : Cancelar a criação da categoria.

## Tecnologias Utilizadas

Este projeto foi desenvolvido em Rust e utiliza as seguintes dependências principais:
- [ratatui](https://crates.io/crates/ratatui) - Para renderização da interface no terminal (TUI).
- [crossterm](https://crates.io/crates/crossterm) - Para manipulação do terminal e captura de eventos de teclado.
- [serde](https://crates.io/crates/serde) e [serde_json](https://crates.io/crates/serde_json) - Para serialização e salvamento dos dados (`tasks.json`).
- [chrono](https://crates.io/crates/chrono) - Para manipulação de datas e horários (criação e conclusão de tarefas).

## Licença

Este projeto é de código aberto e está disponível para uso e modificação.
