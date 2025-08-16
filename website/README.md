# riglr Website

This folder contains the landing page for riglr.

## Structure

```
website/
├── index.html    # Landing page
├── logo-h.png    # Horizontal logo
└── README.md     # This file
```

## Deployment

The landing page is automatically deployed to GitHub Pages along with the documentation via the `.github/workflows/deploy-docs.yml` workflow.

### URLs after deployment:
- `/` - Landing page (from `website/index.html`)
- `/docs/` - Documentation (from `docs/book/`)

## Local Development

To view the landing page locally:
```bash
cd website
python3 -m http.server 8000
# Visit http://localhost:8000
```

## Updating

When updating the landing page:
1. Edit `website/index.html`
2. Test locally
3. Commit and push
4. GitHub Actions will automatically deploy to GitHub Pages