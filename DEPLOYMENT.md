# DencapsBI Platform - Deployment Guide

## Prerequisites Checklist

Before deploying DencapsBI, ensure you have:

- [ ] Rust (latest stable) installed
- [ ] Node.js (v18+) and npm installed
- [ ] Docker and Docker Compose installed
- [ ] MongoDB accessible (via Docker or remote)
- [ ] Redis accessible (via Docker or remote)
- [ ] LM Studio with GPT-OSS-20B model (or compatible AI model)
- [ ] SSL certificates (for production)

## Local Development Setup

### 1. Start Infrastructure

```bash
# Start MongoDB and Redis
docker-compose up -d

# Verify services are running
docker-compose ps
```

### 2. Configure Backend

```bash
cd backend
cp .env.example .env

# Edit .env with your settings
nano .env
```

Required environment variables:
- `MONGODB_URI` - MongoDB connection string
- `REDIS_URI` - Redis connection string
- `JWT_SECRET` - Secure random string (use `openssl rand -base64 32`)
- `LM_STUDIO_API_URL` - LM Studio API endpoint
- `LM_STUDIO_MODEL_NAME` - Model name (default: GPT-OSS-20B)

### 3. Start LM Studio

1. Open LM Studio application
2. Load GPT-OSS-20B model (or your preferred model)
3. Start local server on port 1234
4. Test API: `curl http://localhost:1234/v1/models`

### 4. Build and Run Backend

```bash
cd backend

# Development
cargo run

# Production
cargo build --release
./target/release/dencapsbi-backend
```

The backend will be available at `http://localhost:8080`

### 5. Build and Run Frontend

```bash
cd frontend

# Install dependencies
npm install

# Development
npm start
# Access at http://localhost:4200

# Production build
npm run build
# Deploy dist/ folder to your web server
```

## Production Deployment

### Backend Deployment

1. **Build Release Binary:**
```bash
cd backend
cargo build --release
```

2. **Configure Production Environment:**
```bash
# Create production .env file (only used if RUST_ENV != production)
cp .env.example .env.production

# Set production environment variables (recommended approach)
export RUST_ENV=production  # Prevents loading .env file
export RUST_LOG=info
export SERVER_HOST=0.0.0.0
export SERVER_PORT=8080
export MONGODB_URI="mongodb://production-mongodb:27017"
export REDIS_URI="redis://production-redis:6379"
export JWT_SECRET="your-very-secure-secret-key"
export LM_STUDIO_API_URL="http://ai-server:1234"
export CORS_ALLOWED_ORIGINS="https://yourdomain.com"
```

3. **Deploy Binary:**
```bash
# Copy binary to production server
scp target/release/dencapsbi-backend user@server:/opt/dencapsbi/

# Set up systemd service (example)
sudo nano /etc/systemd/system/dencapsbi.service
```

Example systemd service:
```ini
[Unit]
Description=DencapsBI Backend Server
After=network.target

[Service]
Type=simple
User=dencapsbi
WorkingDirectory=/opt/dencapsbi
Environment="RUST_ENV=production"
EnvironmentFile=/opt/dencapsbi/.env
ExecStart=/opt/dencapsbi/dencapsbi-backend
Restart=always

[Install]
WantedBy=multi-user.target
```

4. **Start Service:**
```bash
sudo systemctl enable dencapsbi
sudo systemctl start dencapsbi
sudo systemctl status dencapsbi
```

### Frontend Deployment

1. **Update Production Configuration:**
```typescript
// src/environments/environment.prod.ts
export const environment = {
  production: true,
  apiUrl: 'https://api.yourdomain.com/api'
};
```

2. **Build for Production:**
```bash
cd frontend
npm run build
```

3. **Deploy to Web Server:**

**Option A: Nginx**
```nginx
server {
    listen 80;
    server_name yourdomain.com;

    root /var/www/dencapsbi/dist/dencapsbi-frontend;
    index index.html;

    location / {
        try_files $uri $uri/ /index.html;
    }

    location /api {
        proxy_pass http://backend-server:8080;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_cache_bypass $http_upgrade;
    }
}
```

**Option B: Apache**
```apache
<VirtualHost *:80>
    ServerName yourdomain.com
    DocumentRoot /var/www/dencapsbi/dist/dencapsbi-frontend

    <Directory /var/www/dencapsbi/dist/dencapsbi-frontend>
        Options -Indexes +FollowSymLinks
        AllowOverride All
        Require all granted
        FallbackResource /index.html
    </Directory>

    ProxyPass /api http://backend-server:8080/api
    ProxyPassReverse /api http://backend-server:8080/api
</VirtualHost>
```

### Database Setup

**MongoDB Indexes:**
```javascript
// Connect to MongoDB
use dencapsbi

// User indexes
db.users.createIndex({ "email": 1 }, { unique: true })
db.users.createIndex({ "tenant_id": 1 })

// Project indexes
db.projects.createIndex({ "tenant_id": 1 })
db.projects.createIndex({ "owner_id": 1 })

// Query indexes
db.analytics_queries.createIndex({ "project_id": 1 })
db.analytics_queries.createIndex({ "user_id": 1 })
```

### SSL/TLS Configuration

**Using Let's Encrypt with Certbot:**
```bash
sudo certbot --nginx -d yourdomain.com
sudo certbot --nginx -d api.yourdomain.com
```

## Monitoring and Logs

### Backend Logs
```bash
# View logs
sudo journalctl -u dencapsbi -f

# Check specific errors
sudo journalctl -u dencapsbi --since today | grep ERROR
```

### Application Logs
The backend uses `env_logger`. Set `RUST_LOG` environment variable:
- `error` - Only errors
- `warn` - Warnings and errors
- `info` - Informational messages (recommended for production)
- `debug` - Detailed debugging
- `trace` - Very verbose debugging

### Health Checks

Create a health check endpoint or monitor:
```bash
# Check backend is responding
curl http://localhost:8080/api/auth/login -I

# Check MongoDB connection
mongo --eval "db.adminCommand('ping')"

# Check Redis connection
redis-cli ping
```

## Security Checklist

- [ ] Change default JWT_SECRET
- [ ] Use strong passwords for databases
- [ ] Enable SSL/TLS for all connections
- [ ] Configure firewall rules
- [ ] Set up rate limiting
- [ ] Regular security updates
- [ ] Backup database regularly
- [ ] Monitor authentication failures
- [ ] Implement IP whitelisting for admin access
- [ ] Use environment variables for all secrets
- [ ] Enable MongoDB authentication
- [ ] Enable Redis authentication
- [ ] Rotate JWT secrets periodically

## Backup Strategy

### MongoDB Backup
```bash
# Daily backup
mongodump --uri="mongodb://localhost:27017/dencapsbi" --out=/backups/$(date +%Y%m%d)

# Restore from backup
mongorestore --uri="mongodb://localhost:27017/dencapsbi" /backups/20240107
```

### Redis Backup
```bash
# Manual backup
redis-cli SAVE

# Configure automatic snapshots in redis.conf
save 900 1      # Save if at least 1 key changed in 900 seconds
save 300 10     # Save if at least 10 keys changed in 300 seconds
save 60 10000   # Save if at least 10000 keys changed in 60 seconds
```

## Scaling Considerations

### Horizontal Scaling
- Use load balancer (Nginx, HAProxy) for multiple backend instances
- Session data stored in Redis (shared across instances)
- MongoDB replica set for high availability

### Vertical Scaling
- Increase server resources (CPU, RAM)
- Optimize MongoDB indexes
- Tune Redis memory limits
- Adjust connection pools

## Troubleshooting

### Backend won't start
- Check environment variables are set
- Verify MongoDB and Redis connections
- Check port 8080 is not in use
- Review logs for specific errors

### Frontend can't connect to backend
- Verify CORS settings in backend
- Check API URL in frontend environment
- Ensure backend is accessible
- Check network/firewall rules

### AI queries failing
- Verify LM Studio is running
- Check LM_STUDIO_API_URL is correct
- Ensure model is loaded in LM Studio
- Check LM Studio logs for errors

### Database connection issues
- Verify MongoDB URI format
- Check MongoDB is running
- Verify authentication credentials
- Check network connectivity

## Performance Tuning

### Backend
- Adjust database connection pool size
- Configure Redis connection pool
- Set appropriate worker threads in Actix Web
- Enable response compression

### Frontend
- Enable Angular production mode
- Use lazy loading for routes
- Implement caching strategies
- Optimize bundle sizes

### Database
- Create appropriate indexes
- Monitor slow queries
- Set up query profiling
- Configure connection pooling

## Support and Maintenance

### Regular Maintenance Tasks
- [ ] Update dependencies weekly
- [ ] Review and rotate logs monthly
- [ ] Test backup restoration quarterly
- [ ] Security audit quarterly
- [ ] Performance review quarterly
- [ ] Update documentation as needed

### Update Process
1. Test updates in staging environment
2. Backup production database
3. Deploy backend updates
4. Deploy frontend updates
5. Run smoke tests
6. Monitor for issues

## Contact and Support

For issues and questions:
- GitHub Issues: https://github.com/ayush-parida/bi.dencaps.com/issues
- Documentation: README.md and API_DOCUMENTATION.md
