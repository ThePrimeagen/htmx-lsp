Allows setting a different target to apply the loading states. The attribute value can be any valid CSS selector. The example below disables the submit button and shows the loading state when the form is submitted.

Example:

<form hx-post="/save"
  data-loading-target="#loading"
  data-loading-class-remove="hidden">

  <button type="submit" data-loading-disable>Submit</button>

</form>

<div id="loading" class="hidden">Loading ...</div>
