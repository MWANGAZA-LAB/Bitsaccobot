# BitSacco WhatsApp Bot

A secure and reliable WhatsApp bot built in Rust for BitSacco users to access savings and Bitcoin services directly through WhatsApp.

## 🚀 Features

- **Secure Messaging**: End-to-end protection of user data and financial transactions
- **BitSacco Integration**: Seamless communication with BitSacco.com backend API
- **Bitcoin Services**: Real-time BTC price updates and balance management
- **Chama Management**: View and manage savings groups
- **Transaction Processing**: Deposit, withdrawal, and transfer capabilities
- **High Performance**: Built with Rust for maximum speed and reliability
- **Production Ready**: Dockerized deployment with CI/CD pipeline

## 📋 Prerequisites

- Rust 1.75+ 
- Docker and Docker Compose
- WhatsApp Cloud API access
- BitSacco.com API access

## 🛠️ Installation

### 1. Clone the Repository

```bash
git clone https://github.com/bitsacco/whatsapp-bot.git
cd whatsapp-bot
```

### 2. Environment Configuration

Copy the example environment file and configure your settings:

```bash
cp env.example .env
```

Edit `.env` with your actual credentials:

```env
# WhatsApp Cloud API Configuration
WHATSAPP_ACCESS_TOKEN=your_whatsapp_access_token_here
WHATSAPP_PHONE_NUMBER_ID=your_phone_number_id_here
WHATSAPP_WEBHOOK_VERIFY_TOKEN=your_webhook_verify_token_here

# BitSacco API Configuration
BITSACCO_API_BASE_URL=https://api.bitsacco.com
BITSACCO_API_TOKEN=your_bitsacco_api_token_here

# Server Configuration
SERVER_HOST=0.0.0.0
SERVER_PORT=8080
RUST_LOG=info

# Security Configuration
RATE_LIMIT_REQUESTS_PER_MINUTE=60
MAX_MESSAGE_LENGTH=4096

# BTC Service Configuration
BTC_API_BASE_URL=https://api.coingecko.com/api/v3
BTC_API_KEY=your_btc_api_key_here
```

### 3. Development Setup

```bash
# Install dependencies
cargo build

# Run tests
cargo test

# Run the application
cargo run
```

### 4. Docker Deployment

```bash
# Build and run with Docker Compose
docker-compose up --build

# Or build the Docker image manually
docker build -t bitsacco-whatsapp-bot .
docker run -p 8080:8080 --env-file .env bitsacco-whatsapp-bot
```

## 🤖 Bot Commands

The bot supports the following commands:

| Command | Description | Example |
|---------|-------------|---------|
| `help` | Show help message | `help` |
| `balance` | Check savings and BTC balance | `balance` |
| `savings` | View detailed savings information | `savings` |
| `chama` | View chama groups | `chama` |
| `btc` | Get current Bitcoin price | `btc` |
| `deposit <amount> <currency>` | Make a deposit | `deposit 100 USD` |
| `withdraw <amount> <currency>` | Make a withdrawal | `withdraw 50 KES` |
| `transfer <amount> <currency> <phone>` | Transfer to another user | `transfer 25 USD +254712345678` |

## 🔧 API Endpoints

### Webhook Endpoint
- **POST** `/webhook` - Receives WhatsApp messages and webhook verification

### Send Message
- **POST** `/send` - Send WhatsApp messages programmatically

### Health Check
- **GET** `/health` - System health and service status

## 🧪 Testing

### Unit Tests
```bash
cargo test
```

### Integration Tests
```bash
cargo test --test integration_tests
```

### Load Testing
```bash
cargo bench
```

### Security Testing
```bash
cargo audit
cargo deny check
```

## 🚀 Deployment

### GitHub Actions CI/CD

The project includes a comprehensive CI/CD pipeline that:

1. **Tests**: Runs unit, integration, and security tests
2. **Builds**: Creates optimized Docker images
3. **Deploys**: Automatically deploys to staging and production
4. **Monitors**: Includes health checks and notifications

### Manual Deployment

#### Railway
```bash
# Install Railway CLI
npm install -g @railway/cli

# Login and deploy
railway login
railway deploy
```

#### Docker
```bash
# Build production image
docker build -t bitsacco-whatsapp-bot:latest .

# Run with environment variables
docker run -d \
  --name bitsacco-bot \
  -p 8080:8080 \
  --env-file .env \
  bitsacco-whatsapp-bot:latest
```

## 🔒 Security Features

- **Input Validation**: All webhook requests are validated and sanitized
- **Rate Limiting**: Configurable rate limiting to prevent abuse
- **HTTPS Only**: Secure communication with backend services
- **No Local Storage**: Sensitive data is not stored locally
- **Audit Logging**: Comprehensive logging with sensitive data redacted
- **Dependency Audits**: Regular security audits of dependencies

## 📊 Monitoring

### Health Checks
The bot provides comprehensive health monitoring:

```bash
curl http://localhost:8080/health
```

Response:
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

### Logging
Structured logging with configurable levels:

```bash
# Set log level
export RUST_LOG=debug

# View logs
docker logs bitsacco-whatsapp-bot
```

## 🏗️ Architecture

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   WhatsApp      │    │   BitSacco       │    │   BTC Service   │
│   Cloud API     │◄──►│   WhatsApp Bot   │◄──►│   (CoinGecko)   │
└─────────────────┘    └──────────────────┘    └─────────────────┘
                                │
                                ▼
                       ┌──────────────────┐
                       │   BitSacco.com   │
                       │   Backend API    │
                       └──────────────────┘
```

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🆘 Support

- **Documentation**: [https://docs.bitsacco.com](https://docs.bitsacco.com)
- **Issues**: [GitHub Issues](https://github.com/bitsacco/whatsapp-bot/issues)
- **Email**: support@bitsacco.com

## 🗺️ Roadmap

- [ ] Multi-language support
- [ ] Advanced analytics and reporting
- [ ] Integration with more cryptocurrency exchanges
- [ ] Voice message support
- [ ] Advanced chama management features
- [ ] Mobile app integration

---

Built with ❤️ by the BitSacco Team
