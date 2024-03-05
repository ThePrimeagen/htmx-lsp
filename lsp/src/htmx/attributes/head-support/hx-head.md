Use this to override the behaviour of the <head> in the response.

Can be:
- merge: default behaviour
- append: append the elements to the existing <head>
- re-eval: remove and append on every request

Example

As an example, consider the following head tag in an existing document:

<head>
    <link rel="stylesheet" href="https://the.missing.style">
    <link rel="stylesheet" href="/css/site1.css">
    <script src="/js/script1.js"></script>
    <script src="/js/script2.js"></script>
</head>

If htmx receives a request containing this new head tag:

<head>
    <link rel="stylesheet" href="https://the.missing.style">
    <link rel="stylesheet" href="/css/site2.css">
    <script src="/js/script2.js"></script>
    <script src="/js/script3.js"></script>
</head>

Then the following operations will occur:

    <link rel="stylesheet" href="https://the.missing.style"> will be left alone
    <link rel="stylesheet" href="/css/site1.css"> will be removed from the head
    <link rel="stylesheet" href="/css/site2.css"> will be added to the head
    <script src="/js/script1.js"></script> will be removed from the head
    <script src="/js/script2.js"></script> will be left alone
    <script src="/js/script3.js"></script> will be added to the head

The final head element will look like this:

<head>
    <link rel="stylesheet" href="https://the.missing.style">
    <script src="/js/script2.js"></script>
    <link rel="stylesheet" href="/css/site2.css">
    <script src="/js/script3.js"></script>
</head>
