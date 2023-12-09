The hx-prompt attribute allows you to show a prompt before issuing a request. The value of the prompt will be included in the request in the HX-Prompt header.

Here is an example:

<button hx-delete="/account" hx-prompt="Enter your account name to confirm deletion">
  Delete My Account
</button>

Notes

    hx-prompt is inherited and can be placed on a parent element


[HTMX Reference](https://htmx.org/attributes/hx-prompt/)
