Look up a mustache template by ID for the template content which is rendered for each element in the array

Example for an API that returns an array:

<div hx-ext="client-side-templates">
  <button hx-get="https://jsonplaceholder.typicode.com/users"
          hx-swap="innerHTML"
          hx-target="#content"
          mustache-array-template="foo">
    Click Me
  </button>

  <p id="content">Start</p>

  <template id="foo">
    {{#data}}
    <p> {{name}} at {{email}} is with {{company.name}}</p>
    {{/data}}
  </template>
</div>
