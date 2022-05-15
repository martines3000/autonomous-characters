FROM rust:1.60-alpine AS builder
RUN apk add binaryen jq libxcb-dev libxkbcommon-dev musl-dev bash openssl-dev

COPY . /ac
WORKDIR /ac

RUN bash ./web.sh

FROM node:16-alpinež

RUN npm install -g serve

COPY --from=builder /ac/out/ ./
COPY --from=builder /ac/index.html .

CMD ["npx", "serve", "."]  


