FROM node:alpine

WORKDIR /work
COPY ["package.json", "yarn.lock", ".yarnrc.yml", "./"]
COPY ./.yarn ./.yarn
COPY ./packages/emails ./packages/emails

WORKDIR /work/packages/emails
RUN yarn install
RUN yarn build

CMD [ "yarn", "start" ]
