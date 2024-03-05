After an HTML page (or page fragment) is preloaded, also preload the linked image resources.

<div hx-ext="preload">
    <a href="/my-next-page" preload="mouseover" preload-images="true">Next Page</a>
</div>

NOTE:
This does not load images from or run Javascript or CSS content.
