# =============================================================================
# Nexus Client - React Native Development Environment
# =============================================================================
# Note: React Native mobile apps (iOS/Android) should be built using:
#   - Expo EAS: https://expo.dev/eas
#   - Or React Native CLI cloud builds
#
# This Dockerfile is for local development/testing only.

FROM node:20-alpine

WORKDIR /app

# Install system dependencies for React Native
RUN apk add --no-cache \
    python3 \
    make \
    g++ \
    java \
    ruby \
    && rm -rf /var/cache/apk/*

# Install Ruby gems for iOS builds (if needed)
RUN gem install bundler --no-document

# Copy package files
COPY app/client/package*.json ./

# Install npm dependencies
RUN npm install

# Copy source
COPY app/client/ ./

# Expose development port
EXPOSE 8081

# Start Metro bundler for development
CMD ["npm", "start"]
