//define a class extending HTMLElement
class HelloWorld extends HTMLElement {
    connectedCallback () {
      this.innerHTML = 'Hello, World222222'
    }
}

//register the new custom element
customElements.define( 'hello-world', HelloWorld )