The element with this attribute will start preloading the data at the location as soon as the `mousedown` event begins.

Example:

<body hx-ext="preload">
    <h1>What Works</h2>
    <a href="/server/1" preload>WILL BE requested using a standard XMLHttpRequest() and default options (below)</a>
    <button hx-get="/server/2" preload>WILL BE requested with additional htmx headers.</button>

    <h1>What WILL NOT WORK</h1>
    <a href="/server/3">WILL NOT be preloaded because it does not have an explicit "preload" attribute</a>
    <a hx-post="/server/4" preload>WILL NOT be preloaded because it is an HX-POST transaction.</a>
</body>

If other preload triggers need to be used this can be done using
preload="event-name" with some default events

Examples:
<a href="/server/1" preload="mousedown">This will be preloaded when the user begins to click.</a>
<a href="/server/1" preload="mouseover">This will be preloaded when the user's mouse remains over it for more than 100ms.</a>
<body hx-ext="preload">
    <button hx-get="/server" preload="preload:init" hx-target="idLoadMore">Load More</a>
    <div id="idLoadMore">
        Content for this DIV will be preloaded as soon as the page is ready.
        Clicking the button above will swap it into the DOM.
    </div>
</body>
