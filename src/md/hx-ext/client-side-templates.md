This extension supports transforming a JSON request response into HTML via a client-side template before it is swapped into the DOM. Currently three client-side templating engines are supported:

    mustache
    handlebars
    nunjucks

When you add this extension on an element, any element below it in the DOM can use one of three attributes named <template-engine>-template (e.g. mustache-template) with a template ID, and the extension will resolve and render the template the standard way for that template engine:

    mustache - looks a mustache <script> tag up by ID for the template content
    handlebars - looks in the Handlebars.partials collection for a template with that name
    nunjucks - resolves the template by name via `nunjucks.render()

The AJAX response body will be parsed as JSON and passed into the template rendering.

Install
<script src="https://unpkg.com/htmx.org/dist/ext/client-side-templates.js"></script>

Usage
<div hx-ext="client-side-templates">
    <button hx-get="/some_json"
          mustache-template="my-mustache-template">
     Handle with mustache
    </button>
    <button hx-get="/some_json"
          handlebars-template="my-handlebars-template">
     Handle with handlebars
    </button>
    <button hx-get="/some_json"
          nunjucks-template="my-nunjucks-template">
     Handle with nunjucks
    </button>
</div>

Full HTML Example

To use the client side template, you will need to include htmx, the extension, and the rendering engine. Here is an example of this setup for Mustache using a <template> tag.

If you wish to put a template into another file, you can use a directive such as <script src="my-template" id="template-id" type="text/mustache">

<!DOCTYPE html>
<html>
  <head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width">
    <title>JS Bin</title>
    <script src="https://unpkg.com/htmx.org"></script>
    <script src="https://unpkg.com/htmx.org/dist/ext/client-side-templates.js"></script>
    <script src="https://unpkg.com/mustache@latest"></script>
  </head>
  <body>
    <div hx-ext="client-side-templates">
      <button hx-get="https://jsonplaceholder.typicode.com/todos/1"
              hx-swap="innerHTML"
              hx-target="#content"
              mustache-template="foo">
        Click Me
      </button>

      <p id="content">Start</p>

      <template id="foo">
        <p> {% raw %}{{userID}}{% endraw %} and {% raw %}{{id}}{% endraw %} and {% raw %}{{title}}{% endraw %} and {% raw %}{{completed}}{% endraw %}</p>
      </template>
    </div>
  </body>
</html>

[HTMX Reference](https://htmx.org/extensions/client-side-templates/)
