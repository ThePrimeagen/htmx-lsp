This extension adds the Triggering-Event header to requests. The value of the header is a JSON serialized version of the event that triggered the request.

Install
<script src="https://unpkg.com/htmx.org/dist/ext/event-header.js"></script>

Usage
<button hx-ext="event-header">
   Click Me!
</button>
Sends something like this:
Triggering-Event: '{ "isTrusted": false, "htmx-internal-data": { "handled": true }, "screenX": 0

[HTMX Reference](https://htmx.org/extensions/event-header/)
