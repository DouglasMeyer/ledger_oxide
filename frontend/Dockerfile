

FROM node:24
WORKDIR /app
COPY package*.json ./
RUN npm install
COPY . .
RUN npx playwright install-deps
RUN npx playwright install
EXPOSE 5173
CMD ["npm", "run", "dev"]
