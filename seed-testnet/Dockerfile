# Dockerfile

FROM node:14-alpine
WORKDIR /app
COPY package.json ./
COPY tsconfig.json ./
COPY src ./src
COPY sa.json ./
ENV GOOGLE_APPLICATION_CREDENTIALS=./sa.json
RUN ls -a
RUN npm install
CMD ["npm", "start"]