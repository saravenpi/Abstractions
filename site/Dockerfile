FROM nginx:1.29-alpine

COPY nginx.conf /etc/nginx/conf.d/default.conf
COPY security-headers.conf /etc/nginx/security-headers.conf
COPY index.html style.css app.js reveal.js copy.js roadmap.js roadmap.css server.sh og.png grain.png favicon.png apple-touch-icon.png /usr/share/nginx/html/
COPY fonts /usr/share/nginx/html/fonts
COPY licenses /usr/share/nginx/html/licenses
COPY server /usr/share/nginx/html/server
COPY roadmap /usr/share/nginx/html/roadmap

EXPOSE 80

HEALTHCHECK --interval=30s --timeout=5s --retries=3 CMD wget -q -O /dev/null http://127.0.0.1/healthz || exit 1
