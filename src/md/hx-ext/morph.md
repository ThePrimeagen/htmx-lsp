Idiomorph is a javascript library for morphing one DOM tree to another. It is inspired by other libraries that pioneered this functionality:

    morphdom - the original DOM morphing library
    nanomorph - an updated take on morphdom

Both morphdom and nanomorph use the id property of a node to match up elements within a given set of sibling nodes. When an id match is found, the existing element is not removed from the DOM, but is instead morphed in place to the new content. This preserves the node in the DOM, and allows state (such as focus) to be retained.

However, in both these algorithms, the structure of the children of sibling nodes is not considered when morphing two nodes: only the ids of the nodes are considered. This is due to performance: it is not feasible to recurse through all the children of siblings when matching things up.

Install
<script src="https://unpkg.com/idiomorph/dist/idiomorph-ext.min.js"></script>

Usage
<div hx-ext="morph">

    <button hx-get="/example" hx-swap="morph:innerHTML">
        Morph My Inner HTML
    </button>

    <button hx-get="/example" hx-swap="morph:outerHTML">
        Morph My Outer HTML
    </button>

    <button hx-get="/example" hx-swap="morph">
        Morph My Outer HTML
    </button>

</div>

[Idiomorph Reference](https://github.com/bigskysoftware/idiomorph)
