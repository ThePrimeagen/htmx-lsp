This extension allows you to use the morphdom library as the swapping mechanism in htmx.
The morphdom library does not support morph element to multiple elements. If the result of hx-select is more than one element, it will pick the first one.

Install
<script src="https://unpkg.com/htmx.org/dist/ext/morphdom-swap.js"></script>

Usage
<header>
  <script src="lib/morphdom-umd.js"></script> <!-- include the morphdom library -->
</header>
<body hx-ext="morphdom-swap">
   <button hx-swap="morphdom">This button will be swapped with morphdom!</button>
</body>

[HTMX Reference](https://htmx.org/extensions/morphdom-swap/)
