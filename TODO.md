# TODO
## Compiler
1. <strike>Token representaiton, in-memory source file representation.</strike>
2. <strike>Implement facility for lexical analysis.</strike>
3. <strike>AST representatio.n</strike>
4. <strike>Design parser.</strike>
5. Design diagnostic subsystem.
6. MVP parser.
7. Semantic analysis
8. Lowering
9. CLI

## Playground
A web application demonstrating the project.

1. A text-editor view demonstrating lexical and syntactic analysis. 
  1. Semitransprent rectangles denoting AST nodes are overlayed onto the source text.
  2. Underlines denoting token boundaries are overlayed onto the source text.
2. Run buttom which compiles and executes the source text.

  We could send the source text over to the server, compile and execute on the server, collect stdout,
  and send it back to the client over HTTP. (Easier)

  Write an interpreter backend, compile the project to WASM, then compile and execute in browser.
  (Harder).

