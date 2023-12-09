hx-delete

The hx-delete attribute will cause an element to issue a DELETE to the specified URL and swap the HTML into the DOM using a swap strategy:

<button hx-delete="/account" hx-target="body">
  Delete Your Account
</button>
This example will cause the button to issue a DELETE to /account and swap the returned HTML into the innerHTML of the body.

Notes
hx-delete is not inherited
You can control the target of the swap using the hx-target attribute
You can control the swap strategy by using the hx-swap attribute
You can control what event triggers the request with the hx-trigger attribute
You can control the data submitted with the request in various ways, documented here: Parameters
To remove the element following a successful DELETE, return a 200 status code with an empty body; if the server responds with a 204, no swap takes place, documented here: Requests & Responses

[HTMX Reference](https://htmx.org/attributes/hx-delete/)
