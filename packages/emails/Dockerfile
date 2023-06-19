FROM node:alpine

WORKDIR /work
COPY ["package.json", "yarn.lock", ".yarnrc.yml", "./"]
COPY ./.yarn ./.yarn
COPY ./packages/emails ./packages/emails
RUN yarn install

WORKDIR /work/packages/emails
RUN yarn worker:build

CMD [ "yarn", "worker:start" ]
