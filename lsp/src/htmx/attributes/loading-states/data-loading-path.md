Allows filtering the processing of loading states only for specific requests based on the request path.

<form hx-post="/save">
  <button type="submit" data-loading-disable data-loading-path="/save">Submit</button>
</form>

You can place the data-loading-path attribute directly on the loading state element, or in any parent element.

<form hx-post="/save" data-loading-path="/save">
  <button type="submit" data-loading-disable>Submit</button>
</form>
