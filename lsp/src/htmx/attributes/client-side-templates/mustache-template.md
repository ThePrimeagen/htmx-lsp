Look up a mustache template by ID for the template content

Example:

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
