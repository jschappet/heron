# Adding a document type

The `/admin/content/` dropdown is defined separately in each website. Heron defines the accepted type names and supplies the editor's field schema. Keep all three in sync using the same lowercase key, such as `results`.

## 1. Update Heron's DocType

In [`src/types/mod.rs`](../src/types/mod.rs), add the new type to:

- The `DocType` enum (for example, `Results`).
- `meta()` (for example, `DocType::Results => ("results", "Results")`).
- `all()` and `list()`.
- The `FromSql` mapping (for example, `"results" => Ok(DocType::Results)`).

Skip entries that already exist. Build and deploy the updated Heron binary to each affected server. No database migration is needed for a new type that uses existing draft columns and metadata.

## 2. Update doc_schema.json on each Heron server

Add an entry inside the top-level `types` object in [`doc_schema.json`](../doc_schema.json). Keep the repository copy and each deployed server's copy in sync. For example:

```json
"results": {
  "label": "Results",
  "has_markdown": true,
  "fields": [
    {
      "label": "Title",
      "key": "title",
      "type": "text",
      "required": true
    },
    {
      "label": "Description",
      "key": "description",
      "type": "textarea"
    },
    {
      "label": "Author",
      "key": "author",
      "type": "text"
    },
    {
      "label": "Tags",
      "key": "tags",
      "type": "text",
      "storage": "meta"
    },
    {
      "label": "Date",
      "key": "date",
      "type": "date",
      "storage": "meta"
    },
    {
      "label": "Images",
      "key": "images",
      "type": "images",
      "storage": "meta"
    },
    {
      "label": "Body Markdown",
      "key": "body_md",
      "type": "markdown",
      "required": true
    }
  ]
}
```

This is an object entry, not a complete JSON document; add a comma between it and adjacent entries. Use `"storage": "meta"` for additional fields that do not correspond to existing draft columns.

Heron reads `./doc_schema.json` relative to its runtime working directory and caches it for the process lifetime. **Restart Heron after updating the file.** The current `deployToProd.sh` does not copy this file, so ensure it is deployed separately. The schema is shared across sites served by that Heron instance.

## 3. Update each site's content admin template

Add the key to `contentAdmin().docTypes` in each site where the type should appear. Locate the template by its `/admin/content/` permalink; the source path varies:

| Site | Source file within its repository |
| --- | --- |
| ReVillage Society | `src/admin-content-7f8a9b2d/content.njk` |
| Regenerate Skagit | `src/content/admin/content.njk` |

If another site uses `src/admin/content.njk`, update that file instead. Edit the source template, not a dated backup or generated `dist/` file.

```javascript
function contentAdmin() {
  return {
    view: 'list', // default to GitHub-style
    drafts: [],
    newDocument: false,
    docTypes: [
      'organization', 'event', 'post', 'idea', 'results'
    ],
    // ...remaining properties and methods...
  };
}
```

This array supplies both the filter dropdown and the new-document choices. It is not populated from Heron's `doc_types` API response and is not a server-side permission rule.

## 4. Rebuild and verify

Rebuild and deploy each changed website using its normal workflow (ReVillage: `npm run build`). Sign in and open `/admin/content/`, then:

1. Confirm the new type appears in the filter and new-document choices.
2. Create a draft and confirm the expected editor fields appear.
3. Save, reopen, and filter for that draft to verify the type and fields persist.

If fields are missing, check the `draft_doc_schema` response (`GET /api/admin/drafts/doc_schema`), the server's runtime schema file, and whether Heron was restarted. If saving rejects the type, check that the deployed binary includes its `DocType` mappings.
