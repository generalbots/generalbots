# sample-node-app — container image for incus (dev + prod).
# No build step needed: the app is a single dependency-free server.js.
FROM node:20-alpine

ENV NODE_ENV=production \
    APP_NAME=sample-node-app \
    APP_VERSION=1.0.0 \
    PORT=8080

WORKDIR /app

COPY package.json server.js ./

EXPOSE 8080

# Run as a non-root user.
RUN addgroup -S app && adduser -S app -G app \
 && chown -R app:app /app
USER app

CMD ["node", "server.js"]
