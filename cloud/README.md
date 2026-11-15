# Murmur Cloud relay

Lets every Murmur download use fast online prompt mode and clean-up with no
setup. The Groq key is stored as a Cloudflare secret, never in the app.

Deploy (free Cloudflare account):

    npx wrangler login
    npx wrangler secret put GROQ_API_KEY
    npx wrangler deploy

Then set the repository variable `MURMUR_CLOUD_URL` to the printed
`https://murmur-cloud.<name>.workers.dev` address and rebuild the app.
