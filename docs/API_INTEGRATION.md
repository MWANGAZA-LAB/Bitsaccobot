# API Integration Guide

This document provides detailed information about integrating with the BitSacco WhatsApp Bot API.

## Overview

The BitSacco WhatsApp Bot provides a RESTful API for managing WhatsApp interactions and integrating with the BitSacco backend services.

## Base URL

```
Production: https://api.bitsacco.com/whatsapp-bot
Staging: https://staging-api.bitsacco.com/whatsapp-bot
Local: http://localhost:8080
```

## Authentication

All API requests require authentication using Bearer tokens:

```bash
Authorization: Bearer YOUR_API_TOKEN
```

## Endpoints

### 1. Webhook Endpoint

**POST** `/webhook`

Receives WhatsApp webhook events and handles message processing.

#### Query Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `hub.mode` | string | Yes | Webhook verification mode |
| `hub.challenge` | string | Yes | Challenge string for verification |
| `hub.verify_token` | string | Yes | Verification token |

#### Request Body

```json
{
  "object": "whatsapp_business_account",
  "entry": [
    {
      "id": "WHATSAPP_BUSINESS_ACCOUNT_ID",
      "changes": [
        {
          "value": {
            "messaging_product": "whatsapp",
            "metadata": {
              "display_phone_number": "15551234567",
              "phone_number_id": "PHONE_NUMBER_ID"
            },
            "contacts": [
              {
                "profile": {
                  "name": "John Doe"
                },
                "wa_id": "15551234567"
              }
            ],
            "messages": [
              {
                "from": "15551234567",
                "id": "wamid.xxx",
                "timestamp": "1234567890",
                "text": {
                  "body": "Hello, I need help with my savings"
                },
                "type": "text"
              }
            ]
          },
          "field": "messages"
        }
      ]
    }
  ]
}
```

#### Response

**Success (200)**
```
OK
```

**Error (400)**
```json
{
  "error": "Invalid webhook payload",
  "status": 400
}
```

### 2. Send Message

**POST** `/send`

Send a WhatsApp message programmatically.

#### Request Body

```json
{
  "to": "+254712345678",
  "message": "Hello! Your savings balance is 1,000 KES."
}
```

#### Response

**Success (200)**
```json
{
  "messaging_product": "whatsapp",
  "contacts": [
    {
      "input": "+254712345678",
      "wa_id": "+254712345678"
    }
  ],
  "messages": [
    {
      "id": "wamid.xxx"
    }
  ]
}
```

**Error (400)**
```json
{
  "error": "Message too long",
  "status": 400
}
```

### 3. Health Check

**GET** `/health`

Check the health status of the bot and its dependencies.

#### Response

**Success (200)**
```json
{
  "status": "ok",
  "timestamp": "2023-12-01T12:00:00Z",
  "version": "0.1.0",
  "services": {
    "whatsapp": "healthy",
    "bitsacco": "healthy",
    "btc": "healthy"
  }
}
```

## Bot Commands

The bot processes natural language commands and structured commands:

### Natural Language Commands

| Command | Description | Example |
|---------|-------------|---------|
| "help" | Show help message | User: "help" |
| "balance" | Check account balance | User: "balance" |
| "savings" | View savings details | User: "savings" |
| "chama" | View chama groups | User: "chama" |
| "bitcoin" | Get BTC price | User: "bitcoin" |

### Structured Commands

| Command | Format | Description | Example |
|---------|--------|-------------|---------|
| Deposit | `deposit <amount> <currency>` | Make a deposit | `deposit 100 USD` |
| Withdraw | `withdraw <amount> <currency>` | Make a withdrawal | `withdraw 50 KES` |
| Transfer | `transfer <amount> <currency> <phone>` | Transfer to user | `transfer 25 USD +254712345678` |

## Error Handling

The API uses standard HTTP status codes and returns structured error responses:

### Error Response Format

```json
{
  "error": "Error description",
  "status": 400
}
```

### Common Error Codes

| Status | Description |
|--------|-------------|
| 400 | Bad Request - Invalid input |
| 401 | Unauthorized - Invalid token |
| 404 | Not Found - Resource not found |
| 429 | Too Many Requests - Rate limit exceeded |
| 500 | Internal Server Error |
| 502 | Bad Gateway - External service error |

## Rate Limiting

The API implements rate limiting to prevent abuse:

- **Default**: 60 requests per minute per IP
- **Headers**: Rate limit information is included in response headers
- **Exceeded**: Returns 429 status code when limit is exceeded

## Webhook Security

### Verification Process

1. WhatsApp sends a GET request to your webhook URL with verification parameters
2. The bot verifies the `hub.verify_token` matches your configured token
3. If valid, returns the `hub.challenge` string
4. WhatsApp confirms the webhook is properly configured

### Signature Verification

All webhook payloads include a signature header for verification:

```
X-Hub-Signature-256: sha256=<signature>
```

## Integration Examples

### Node.js Example

```javascript
const axios = require('axios');

const botAPI = axios.create({
  baseURL: 'https://api.bitsacco.com/whatsapp-bot',
  headers: {
    'Authorization': 'Bearer YOUR_API_TOKEN',
    'Content-Type': 'application/json'
  }
});

// Send a message
async function sendMessage(to, message) {
  try {
    const response = await botAPI.post('/send', {
      to,
      message
    });
    console.log('Message sent:', response.data);
  } catch (error) {
    console.error('Error sending message:', error.response.data);
  }
}

// Check health
async function checkHealth() {
  try {
    const response = await botAPI.get('/health');
    console.log('Bot status:', response.data);
  } catch (error) {
    console.error('Health check failed:', error.response.data);
  }
}
```

### Python Example

```python
import requests

class BitSaccoBotAPI:
    def __init__(self, base_url, api_token):
        self.base_url = base_url
        self.headers = {
            'Authorization': f'Bearer {api_token}',
            'Content-Type': 'application/json'
        }
    
    def send_message(self, to, message):
        url = f"{self.base_url}/send"
        data = {"to": to, "message": message}
        
        response = requests.post(url, json=data, headers=self.headers)
        return response.json()
    
    def health_check(self):
        url = f"{self.base_url}/health"
        response = requests.get(url, headers=self.headers)
        return response.json()

# Usage
bot = BitSaccoBotAPI('https://api.bitsacco.com/whatsapp-bot', 'YOUR_API_TOKEN')
result = bot.send_message('+254712345678', 'Hello from BitSacco!')
print(result)
```

### cURL Examples

```bash
# Send a message
curl -X POST https://api.bitsacco.com/whatsapp-bot/send \
  -H "Authorization: Bearer YOUR_API_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "to": "+254712345678",
    "message": "Your savings balance is 1,000 KES"
  }'

# Health check
curl -X GET https://api.bitsacco.com/whatsapp-bot/health \
  -H "Authorization: Bearer YOUR_API_TOKEN"

# Webhook verification
curl -X GET "https://api.bitsacco.com/whatsapp-bot/webhook?hub.mode=subscribe&hub.challenge=CHALLENGE&hub.verify_token=YOUR_VERIFY_TOKEN"
```

## Testing

### Test Environment

Use the staging environment for testing:

```bash
export BOT_API_URL="https://staging-api.bitsacco.com/whatsapp-bot"
export BOT_API_TOKEN="your_staging_token"
```

### Webhook Testing

Use ngrok for local webhook testing:

```bash
# Install ngrok
npm install -g ngrok

# Expose local server
ngrok http 8080

# Use the ngrok URL as your webhook URL in WhatsApp Business API
```

## Monitoring and Logging

### Log Levels

Configure logging level via environment variable:

```bash
export RUST_LOG=debug  # debug, info, warn, error
```

### Metrics

The bot exposes metrics for monitoring:

- Message processing time
- API response times
- Error rates
- Active user count

### Alerts

Set up alerts for:

- High error rates
- Service downtime
- Rate limit violations
- Unusual message patterns

## Best Practices

1. **Error Handling**: Always implement proper error handling for API calls
2. **Rate Limiting**: Respect rate limits and implement exponential backoff
3. **Security**: Use HTTPS for all communications
4. **Monitoring**: Implement health checks and monitoring
5. **Testing**: Test thoroughly in staging environment before production
6. **Documentation**: Keep API documentation updated

## Support

For API support and questions:

- **Email**: api-support@bitsacco.com
- **Documentation**: [https://docs.bitsacco.com](https://docs.bitsacco.com)
- **Status Page**: [https://status.bitsacco.com](https://status.bitsacco.com)
