reference an extension

Tip: To use multiple extensions on one element, seperate them with a comma:
  <button hx-post="/example" hx-ext="debug, json-enc">This Button Uses Two Extensions</button>

by default, extensions are applied to the DOM node where it is invoked, along with all child elements inside of that parent node. If you need to disable an extension somewhere within the DOM tree, you can use the ignore: keyword to stop it from being used.


[HTMX Reference](https://htmx.org/attributes/hx-ext/)
