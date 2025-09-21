// Mobile Navigation Toggle
document.addEventListener('DOMContentLoaded', function() {
    const hamburger = document.querySelector('.hamburger');
    const navMenu = document.querySelector('.nav-menu');
    
    if (hamburger && navMenu) {
        hamburger.addEventListener('click', function() {
            navMenu.classList.toggle('active');
            hamburger.classList.toggle('active');
        });
    }
});

// Smooth scrolling for navigation links
document.querySelectorAll('a[href^="#"]').forEach(anchor => {
    anchor.addEventListener('click', function (e) {
        e.preventDefault();
        const target = document.querySelector(this.getAttribute('href'));
        if (target) {
            target.scrollIntoView({
                behavior: 'smooth',
                block: 'start'
            });
        }
    });
});

// Copy code functionality
function copyCode(button) {
    const codeBlock = button.closest('.code-block');
    const code = codeBlock.querySelector('code');
    const text = code.textContent;
    
    // Create a temporary textarea to copy text
    const textarea = document.createElement('textarea');
    textarea.value = text;
    document.body.appendChild(textarea);
    textarea.select();
    document.execCommand('copy');
    document.body.removeChild(textarea);
    
    // Visual feedback
    const originalIcon = button.innerHTML;
    button.innerHTML = '<i class="fas fa-check"></i>';
    button.style.color = '#10b981';
    
    setTimeout(() => {
        button.innerHTML = originalIcon;
        button.style.color = '';
    }, 2000);
}

// Navbar scroll effect
window.addEventListener('scroll', function() {
    const navbar = document.querySelector('.navbar');
    if (window.scrollY > 100) {
        navbar.style.background = 'rgba(255, 255, 255, 0.98)';
        navbar.style.boxShadow = '0 2px 20px rgba(0, 0, 0, 0.1)';
    } else {
        navbar.style.background = 'rgba(255, 255, 255, 0.95)';
        navbar.style.boxShadow = 'none';
    }
});

// Intersection Observer for animations
const observerOptions = {
    threshold: 0.1,
    rootMargin: '0px 0px -50px 0px'
};

const observer = new IntersectionObserver(function(entries) {
    entries.forEach(entry => {
        if (entry.isIntersecting) {
            entry.target.style.opacity = '1';
            entry.target.style.transform = 'translateY(0)';
        }
    });
}, observerOptions);

// Observe elements for animation
document.addEventListener('DOMContentLoaded', function() {
    const animatedElements = document.querySelectorAll('.feature-card, .command-category, .deployment-card');
    animatedElements.forEach(el => {
        el.style.opacity = '0';
        el.style.transform = 'translateY(30px)';
        el.style.transition = 'opacity 0.6s ease, transform 0.6s ease';
        observer.observe(el);
    });
});

// Typing animation for hero title
function typeWriter(element, text, speed = 100) {
    let i = 0;
    element.innerHTML = '';
    
    function type() {
        if (i < text.length) {
            element.innerHTML += text.charAt(i);
            i++;
            setTimeout(type, speed);
        }
    }
    
    type();
}

// Initialize typing animation when page loads
document.addEventListener('DOMContentLoaded', function() {
    const heroTitle = document.querySelector('.hero-title .gradient-text');
    if (heroTitle) {
        const originalText = heroTitle.textContent;
        setTimeout(() => {
            typeWriter(heroTitle, originalText, 80);
        }, 500);
    }
});

// Phone mockup chat animation
function animateChatMessages() {
    const messages = document.querySelectorAll('.message');
    messages.forEach((message, index) => {
        message.style.opacity = '0';
        message.style.transform = 'translateY(20px)';
        
        setTimeout(() => {
            message.style.transition = 'opacity 0.5s ease, transform 0.5s ease';
            message.style.opacity = '1';
            message.style.transform = 'translateY(0)';
        }, index * 1000 + 2000);
    });
}

// Start chat animation when hero section is visible
const heroObserver = new IntersectionObserver(function(entries) {
    entries.forEach(entry => {
        if (entry.isIntersecting) {
            setTimeout(animateChatMessages, 1000);
            heroObserver.unobserve(entry.target);
        }
    });
}, { threshold: 0.5 });

document.addEventListener('DOMContentLoaded', function() {
    const heroSection = document.querySelector('.hero');
    if (heroSection) {
        heroObserver.observe(heroSection);
    }
});

// Command search functionality
function filterCommands(searchTerm) {
    const commandItems = document.querySelectorAll('.command-item');
    const searchTermLower = searchTerm.toLowerCase();
    
    commandItems.forEach(item => {
        const command = item.querySelector('.command').textContent.toLowerCase();
        const description = item.querySelector('.command-desc').textContent.toLowerCase();
        
        if (command.includes(searchTermLower) || description.includes(searchTermLower)) {
            item.style.display = 'flex';
        } else {
            item.style.display = 'none';
        }
    });
}

// Add search functionality to commands section
document.addEventListener('DOMContentLoaded', function() {
    const commandsSection = document.querySelector('.commands');
    if (commandsSection) {
        const searchInput = document.createElement('input');
        searchInput.type = 'text';
        searchInput.placeholder = 'Search commands...';
        searchInput.style.cssText = `
            width: 100%;
            max-width: 400px;
            margin: 0 auto 30px;
            padding: 12px 16px;
            border: 2px solid #e5e7eb;
            border-radius: 8px;
            font-size: 1rem;
            display: block;
        `;
        
        searchInput.addEventListener('input', function() {
            filterCommands(this.value);
        });
        
        const commandsGrid = commandsSection.querySelector('.commands-grid');
        commandsGrid.parentNode.insertBefore(searchInput, commandsGrid);
    }
});

// Statistics counter animation
function animateCounters() {
    const counters = document.querySelectorAll('.stat-number');
    
    counters.forEach(counter => {
        const target = parseInt(counter.textContent);
        if (!isNaN(target)) {
            let current = 0;
            const increment = target / 50;
            const timer = setInterval(() => {
                current += increment;
                if (current >= target) {
                    counter.textContent = target;
                    clearInterval(timer);
                } else {
                    counter.textContent = Math.floor(current);
                }
            }, 30);
        }
    });
}

// Performance optimization: Debounce scroll events
function debounce(func, wait) {
    let timeout;
    return function executedFunction(...args) {
        const later = () => {
            clearTimeout(timeout);
            func(...args);
        };
        clearTimeout(timeout);
        timeout = setTimeout(later, wait);
    };
}

// Optimized scroll handler
const debouncedScrollHandler = debounce(function() {
    const navbar = document.querySelector('.navbar');
    if (window.scrollY > 100) {
        navbar.style.background = 'rgba(255, 255, 255, 0.98)';
        navbar.style.boxShadow = '0 2px 20px rgba(0, 0, 0, 0.1)';
    } else {
        navbar.style.background = 'rgba(255, 255, 255, 0.95)';
        navbar.style.boxShadow = 'none';
    }
}, 10);

window.addEventListener('scroll', debouncedScrollHandler);

// Lazy loading for images (if any are added later)
function lazyLoadImages() {
    const images = document.querySelectorAll('img[data-src]');
    const imageObserver = new IntersectionObserver((entries, observer) => {
        entries.forEach(entry => {
            if (entry.isIntersecting) {
                const img = entry.target;
                img.src = img.dataset.src;
                img.classList.remove('lazy');
                imageObserver.unobserve(img);
            }
        });
    });

    images.forEach(img => imageObserver.observe(img));
}

// Initialize lazy loading
document.addEventListener('DOMContentLoaded', lazyLoadImages);

// Error handling for failed API calls (if any are added)
window.addEventListener('error', function(e) {
    console.error('JavaScript error:', e.error);
});

// Service Worker registration (for future PWA features)
if ('serviceWorker' in navigator) {
    window.addEventListener('load', function() {
        navigator.serviceWorker.register('/sw.js')
            .then(function(registration) {
                console.log('ServiceWorker registration successful');
            })
            .catch(function(err) {
                console.log('ServiceWorker registration failed');
            });
    });
}

// Keyboard navigation support
document.addEventListener('keydown', function(e) {
    // ESC key to close mobile menu
    if (e.key === 'Escape') {
        const navMenu = document.querySelector('.nav-menu');
        const hamburger = document.querySelector('.hamburger');
        if (navMenu && navMenu.classList.contains('active')) {
            navMenu.classList.remove('active');
            hamburger.classList.remove('active');
        }
    }
});

// Accessibility improvements
document.addEventListener('DOMContentLoaded', function() {
    // Add skip link for keyboard navigation
    const skipLink = document.createElement('a');
    skipLink.href = '#main-content';
    skipLink.textContent = 'Skip to main content';
    skipLink.style.cssText = `
        position: absolute;
        top: -40px;
        left: 6px;
        background: #000;
        color: #fff;
        padding: 8px;
        text-decoration: none;
        z-index: 1000;
        transition: top 0.3s;
    `;
    
    skipLink.addEventListener('focus', function() {
        this.style.top = '6px';
    });
    
    skipLink.addEventListener('blur', function() {
        this.style.top = '-40px';
    });
    
    document.body.insertBefore(skipLink, document.body.firstChild);
    
    // Add main content landmark
    const mainContent = document.querySelector('.hero');
    if (mainContent) {
        mainContent.id = 'main-content';
        mainContent.setAttribute('role', 'main');
    }
});

// Console welcome message
console.log(`
🤖 BitSacco WhatsApp Bot
Built with ❤️ using Rust

GitHub: https://github.com/MWANGAZA-LAB/Bitsaccobot
Website: https://mwanga-lab.github.io/Bitsaccobot

Features:
✅ Personal savings management
✅ Bitcoin price tracking (Coinbase API)
✅ Advanced chama management
✅ Voice message support
✅ Bank-grade security
✅ High performance (Rust)
`);

// Export functions for potential module use
if (typeof module !== 'undefined' && module.exports) {
    module.exports = {
        copyCode,
        filterCommands,
        typeWriter,
        animateChatMessages
    };
}

// Live Bitcoin Price Fetcher
async function fetchBitcoinPrice() {
    try {
        // Fetch USD price
        const usdResponse = await fetch('https://api.coinbase.com/v2/prices/BTC-USD/spot');
        const usdData = await usdResponse.json();
        const usdPrice = parseFloat(usdData.data.amount);
        
        // Fetch KES price (using USD to KES conversion rate)
        // Note: In a real implementation, you'd want to get KES directly or use a proper exchange rate API
        const kesRate = 150; // Approximate USD to KES rate (you might want to fetch this from an API)
        const kesPrice = usdPrice * kesRate;
        
        // Update the DOM
        document.getElementById('btc-price-usd').textContent = `$${usdPrice.toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`;
        document.getElementById('btc-price-kes').textContent = `KES ${kesPrice.toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`;
        
        // Update timestamp
        const now = new Date();
        document.getElementById('price-update-time').textContent = now.toLocaleTimeString('en-US', { 
            hour12: false, 
            hour: '2-digit', 
            minute: '2-digit' 
        });
        
    } catch (error) {
        console.error('Error fetching Bitcoin price:', error);
        document.getElementById('btc-price-usd').textContent = 'Error';
        document.getElementById('btc-price-kes').textContent = 'Error';
        document.getElementById('price-update-time').textContent = 'Failed';
    }
}

// BitSacco WhatsApp Bot Animated Chat
document.addEventListener('DOMContentLoaded', function() {
    // Fetch Bitcoin price on page load
    fetchBitcoinPrice();
    
    // Update Bitcoin price every 30 seconds
    setInterval(fetchBitcoinPrice, 30000);
    const chatMessages = [
        {
            user: "balance",
            bot: "💰 Your BitSacco Balance<br><br>Total Balance: 127,550,000 sats (1,275.50 KES)<br><br>Last updated: 2024-01-15 14:30:25 UTC"
        },
        {
            user: "bitcoin price",
            bot: "₿ Bitcoin Price: $45,230.50 USD<br>24h Change: +2.5% 📈<br>KES: 6,784,575.00 KES"
        },
        {
            user: "deposit 100 KES mpesa",
            bot: "💰 M-Pesa Deposit Initiated!<br><br>Amount: 100.00 KES<br>Transaction ID: TXN_789456<br>Status: pending<br><br>📱 M-Pesa STK Push sent to your phone!<br><br>Please check your phone and enter your M-Pesa PIN to complete the deposit."
        },
        {
            user: "deposit 50 KES lightning",
            bot: "⚡ Lightning Deposit Initiated!<br><br>Amount: 50.00 KES<br>Payment Request: lnbc500n1p0...<br><br>📱 Scan the QR code or copy the payment request to your Lightning wallet to complete the deposit."
        },
        {
            user: "membership",
            bot: "🏛️ BitSacco Membership<br><br>Shares Owned: 150<br>Total Investment: 1,500.00 USD<br><br>Use `buy shares <count>` to purchase more shares.<br>Use `share history` to view your purchase history."
        },
        {
            user: "buy shares 10 mpesa",
            bot: "🎯 Share Purchase Initiated!<br><br>Shares: 10<br>Amount: 100.00 USD<br>Payment Method: mpesa<br>Transaction ID: SHARE_789<br>Status: pending<br><br>Your shares will be added to your account once payment is confirmed."
        },
        {
            user: "chama",
            bot: "🏦 Your Chama Groups<br><br>• Investment Club CH123<br>&nbsp;&nbsp;Members: 15<br>&nbsp;&nbsp;Your Shares: 25<br><br>• Savings Group CH456<br>&nbsp;&nbsp;Members: 8<br>&nbsp;&nbsp;Your Shares: 12"
        },
        {
            user: "shares balance",
            bot: "📊 Your Chama Shares<br><br>• Chama: Investment Club CH123<br>&nbsp;&nbsp;Shares: 25<br>&nbsp;&nbsp;Total Contribution: 2,500.00 USD<br>&nbsp;&nbsp;Last Updated: 2024-01-10<br><br>• Chama: Savings Group CH456<br>&nbsp;&nbsp;Shares: 12<br>&nbsp;&nbsp;Total Contribution: 1,200.00 USD<br>&nbsp;&nbsp;Last Updated: 2024-01-12"
        },
        {
            user: "history",
            bot: "📋 Recent Transactions<br><br>• TXN_001 - 100.00 KES (mpesa)<br>&nbsp;&nbsp;Type: deposit<br>&nbsp;&nbsp;Status: completed<br>&nbsp;&nbsp;Date: 2024-01-15<br><br>• TXN_002 - 50.00 USD (lightning)<br>&nbsp;&nbsp;Type: share_purchase<br>&nbsp;&nbsp;Status: completed<br>&nbsp;&nbsp;Date: 2024-01-14"
        },
        {
            user: "withdraw 200 KES lightning",
            bot: "⚡ Lightning Withdrawal Initiated!<br><br>Amount: 200.00 KES<br>Transaction ID: WTH_456<br>Status: pending<br><br>📱 Withdrawal will be processed via Lightning Network."
        }
    ];

    let currentMessageIndex = 0;
    let isAnimating = false;

    function animateBitSaccoChat() {
        const chatContainer = document.querySelector('.chat-messages');
        if (!chatContainer || isAnimating) return;
        
        isAnimating = true;
        
        // Clear existing messages except welcome message
        const welcomeMessage = chatContainer.querySelector('.message:first-child');
        chatContainer.innerHTML = '';
        if (welcomeMessage) {
            chatContainer.appendChild(welcomeMessage);
        }
        
        const message = chatMessages[currentMessageIndex];
        
        // Add user message
        setTimeout(() => {
            const userMessage = document.createElement('div');
            userMessage.className = 'message user-message';
            userMessage.innerHTML = `<div class="message-content">${message.user}</div>`;
            chatContainer.appendChild(userMessage);
            chatContainer.scrollTop = chatContainer.scrollHeight;
        }, 500);
        
        // Add bot response
        setTimeout(() => {
            const botMessage = document.createElement('div');
            botMessage.className = 'message bot-message';
            botMessage.innerHTML = `<div class="message-content">${message.bot}</div>`;
            chatContainer.appendChild(botMessage);
            chatContainer.scrollTop = chatContainer.scrollHeight;
        }, 1500);
        
        // Move to next message
        setTimeout(() => {
            currentMessageIndex = (currentMessageIndex + 1) % chatMessages.length;
            isAnimating = false;
        }, 3000);
    }

    // Start BitSacco chat animation when page loads
    setTimeout(() => {
        animateBitSaccoChat();
        // Repeat every 4 seconds
        setInterval(animateBitSaccoChat, 4000);
    }, 2000);
});
