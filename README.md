# Bicep Docs

A documentation generator for Bicep files, similar to terraform-docs.

## Why `bicep-docs`?

That is an excellent question. When I first started looking at automating my Bicep documentation, the main tool I found was [PSDocs for Azure][psdocs-azure]. I had three main complaints with PSDocs, which (admittedly) are related. The first was the lack of [native support for Bicep files][psdocs-106]. This means documenting a Bicep file required compilation to ARM, which led to the other two problems.

Problem two is, honestly minor. Because of the requirement to compile to ARM, documentation required the full Bicep stack. Which is fine when you are deploying the Bicep, but is overkill when you just want to generate/validate the documentation in a pipeline.

Problem three, however, is the one that motivated me to action. Because of the compilation to ARM, many of the nuances of a Bicep template gets lost in the documentation. For example, I like to use a central Bicep module with common types and functions that I'll need frequently. When the Bicep file is compiled to ARM, these functions are fetched and compiled in to the document. This is needed for deployment, but bloats the documenation of my file and also hides the fact of the shared module eco-system. The same problem occurs when referencing local modules.

[psdocs-azure]: https://github.com/Azure/PSDocs.Azure 'PSDocs for Azure'
[psdocs-106]: https://github.com/Azure/PSDocs.Azure/issues/106 'Generate docs for Bicep modules'
