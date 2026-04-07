FROM python:3.11-slim

WORKDIR /app

# Install dependencies
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

# Copy adapter code
COPY adapter/ ./adapter/

# Expose port
EXPOSE 50051

# Run the adapter service
CMD ["python", "-m", "adapter.main"]
