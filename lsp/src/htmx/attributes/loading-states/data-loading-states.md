This attribute is optional and it allows defining a scope for the loading states so only elements within that scope are processed.

Example:
<div data-loading-states>
  <div hx-get=""></div>
  <div data-loading>loading</div>
</div>

<div data-loading-states>
  <div hx-get=""></div>
  <div data-loading>loading</div>
</div>

<form data-loading-states hx-post="">
  <div data-loading>loading</div>
</form>
