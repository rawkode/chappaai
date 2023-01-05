FROM node:18 AS build

WORKDIR /app

COPY package.json ./
COPY package-lock.json ./
RUN npm install

COPY . ./
RUN npm run build

FROM node:18 AS deploy

WORKDIR /app

COPY package.json .
COPY package-lock.json .

RUN npm ci --omit=dev

COPY --from=build /app/dist /app

ENTRYPOINT [ "node" ]
CMD [ "index.js" ]
