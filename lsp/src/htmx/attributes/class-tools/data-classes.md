A data-classes attribute value consists of “runs”, which are separated by an & character. All class operations within a given run will be applied sequentially, with the delay specified.

Within a run, a , character separates distinct class operations.

A class operation is an operation name add, remove, or toggle, followed by a CSS class name, optionally followed by a colon : and a time delay.

<div hx-ext="class-tools">
    <div data-classes="add foo"/> <!-- adds the class "foo" after 100ms -->
    <div class="bar" data-classes="remove bar:1s"/> <!-- removes the class "bar" after 1s -->
    <div class="bar" data-classes="remove bar:1s, add foo:1s"/> <!-- removes the class "bar" after 1s
                                                                then adds the class "foo" 1s after that -->
    <div class="bar" data-classes="remove bar:1s & add foo:1s"/> <!-- removes the class "bar" and adds
                                                                 class "foo" after 1s  -->
    <div data-classes="toggle foo:1s"/> <!-- toggles the class "foo" every 1s -->
</div>
