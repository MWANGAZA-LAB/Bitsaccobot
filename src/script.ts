// TypeScript definitions for BitSacco WhatsApp Bot
interface ChatMessage {
    user: string;
    bot: string;
}

interface BitcoinPrice {
    usd: number;
    kes: number;
    timestamp: string;
}

interface BitcoinPriceResponse {
    data: {
        amount: string;
        base: string;
        currency: string;
    };
}

// Global variables with proper typing
let currentMessageIndex: number = 0;
let isAnimating: boolean = false;

// Chat messages array with proper typing
const chatMessages: ChatMessage[] = [
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

// Utility function to safely get element by ID
function getElementById<T extends HTMLElement>(id: string): T | null {
    return document.getElementById(id) as T | null;
}

// Live Bitcoin Price Fetcher with proper error handling
async function fetchBitcoinPrice(): Promise<void> {
    try {
        // Fetch USD price
        const usdResponse: Response = await fetch('https://api.coinbase.com/v2/prices/BTC-USD/spot');
        
        if (!usdResponse.ok) {
            throw new Error(`HTTP error! status: ${usdResponse.status}`);
        }
        
        const usdData: BitcoinPriceResponse = await usdResponse.json();
        const usdPrice: number = parseFloat(usdData.data.amount);
        
        // Fetch KES price (using USD to KES conversion rate)
        // Note: In a real implementation, you'd want to get KES directly or use a proper exchange rate API
        const kesRate: number = 150; // Approximate USD to KES rate (you might want to fetch this from an API)
        const kesPrice: number = usdPrice * kesRate;
        
        // Update the DOM elements
        const usdElement: HTMLElement | null = getElementById<HTMLElement>('btc-price-usd');
        const kesElement: HTMLElement | null = getElementById<HTMLElement>('btc-price-kes');
        const timeElement: HTMLElement | null = getElementById<HTMLElement>('price-update-time');
        
        if (usdElement) {
            usdElement.textContent = `$${usdPrice.toLocaleString('en-US', { 
                minimumFractionDigits: 2, 
                maximumFractionDigits: 2 
            })}`;
        }
        
        if (kesElement) {
            kesElement.textContent = `KES ${kesPrice.toLocaleString('en-US', { 
                minimumFractionDigits: 2, 
                maximumFractionDigits: 2 
            })}`;
        }
        
        // Update timestamp
        if (timeElement) {
            const now: Date = new Date();
            timeElement.textContent = now.toLocaleTimeString('en-US', { 
                hour12: false, 
                hour: '2-digit', 
                minute: '2-digit' 
            });
        }
        
    } catch (error: unknown) {
        console.error('Error fetching Bitcoin price:', error);
        
        const usdElement: HTMLElement | null = getElementById<HTMLElement>('btc-price-usd');
        const kesElement: HTMLElement | null = getElementById<HTMLElement>('btc-price-kes');
        const timeElement: HTMLElement | null = getElementById<HTMLElement>('price-update-time');
        
        if (usdElement) usdElement.textContent = 'Error';
        if (kesElement) kesElement.textContent = 'Error';
        if (timeElement) timeElement.textContent = 'Failed';
    }
}

// BitSacco WhatsApp Bot Animated Chat with proper typing
function animateBitSaccoChat(): void {
    const chatContainer: HTMLElement | null = document.querySelector('.chat-messages');
    if (!chatContainer || isAnimating) return;
    
    isAnimating = true;
    
    // Clear existing messages except welcome message
    const welcomeMessage: Element | null = chatContainer.querySelector('.message:first-child');
    chatContainer.innerHTML = '';
    if (welcomeMessage) {
        chatContainer.appendChild(welcomeMessage);
    }
    
    const message: ChatMessage = chatMessages[currentMessageIndex];
    
    // Add user message
    setTimeout((): void => {
        const userMessage: HTMLDivElement = document.createElement('div');
        userMessage.className = 'message user-message';
        userMessage.innerHTML = `<div class="message-content">${message.user}</div>`;
        chatContainer.appendChild(userMessage);
        chatContainer.scrollTop = chatContainer.scrollHeight;
    }, 500);
    
    // Add bot response
    setTimeout((): void => {
        const botMessage: HTMLDivElement = document.createElement('div');
        botMessage.className = 'message bot-message';
        botMessage.innerHTML = `<div class="message-content">${message.bot}</div>`;
        chatContainer.appendChild(botMessage);
        chatContainer.scrollTop = chatContainer.scrollHeight;
    }, 1500);
    
    // Move to next message
    setTimeout((): void => {
        currentMessageIndex = (currentMessageIndex + 1) % chatMessages.length;
        isAnimating = false;
        
        // If we've completed all messages, reset to start
        if (currentMessageIndex === 0) {
            // Clear the chat and start fresh
            setTimeout((): void => {
                const chatContainer: HTMLElement | null = document.querySelector('.chat-messages');
                if (chatContainer) {
                    const welcomeMessage: Element | null = chatContainer.querySelector('.message:first-child');
                    chatContainer.innerHTML = '';
                    if (welcomeMessage) {
                        chatContainer.appendChild(welcomeMessage);
                    }
                    
                    // Add a subtle restart indicator
                    const restartMessage: HTMLDivElement = document.createElement('div');
                    restartMessage.className = 'message bot-message';
                    restartMessage.innerHTML = '<div class="message-content">🔄 <em>Demo cycle restarted...</em></div>';
                    chatContainer.appendChild(restartMessage);
                    chatContainer.scrollTop = chatContainer.scrollHeight;
                    
                    // Remove restart message after 2 seconds
                    setTimeout((): void => {
                        if (restartMessage.parentNode) {
                            restartMessage.parentNode.removeChild(restartMessage);
                        }
                    }, 2000);
                }
            }, 1000);
        }
    }, 3000);
}

// Mobile navigation toggle with proper typing
function toggleMobileMenu(): void {
    const navMenu: HTMLElement | null = document.querySelector('.nav-menu');
    const hamburger: HTMLElement | null = document.querySelector('.hamburger');
    
    if (navMenu && hamburger) {
        navMenu.classList.toggle('active');
        hamburger.classList.toggle('active');
    }
}

// Copy code to clipboard function with proper typing
function copyCode(button: HTMLElement): void {
    const codeBlock: HTMLElement | null = button.parentElement?.querySelector('code');
    if (codeBlock) {
        const text: string = codeBlock.textContent || '';
        navigator.clipboard.writeText(text).then((): void => {
            // Visual feedback
            const originalText: string = button.innerHTML;
            button.innerHTML = '<i class="fas fa-check"></i>';
            button.style.color = '#20b2aa';
            
            setTimeout((): void => {
                button.innerHTML = originalText;
                button.style.color = '';
            }, 2000);
        }).catch((error: Error): void => {
            console.error('Failed to copy text: ', error);
        });
    }
}

// Smooth scrolling for navigation links
function smoothScrollTo(targetId: string): void {
    const targetElement: HTMLElement | null = document.getElementById(targetId);
    if (targetElement) {
        targetElement.scrollIntoView({
            behavior: 'smooth',
            block: 'start'
        });
    }
}

// Initialize the application when DOM is loaded
document.addEventListener('DOMContentLoaded', (): void => {
    // Fetch Bitcoin price on page load
    fetchBitcoinPrice();
    
    // Update Bitcoin price every 30 seconds
    setInterval(fetchBitcoinPrice, 30000);
    
    // Mobile menu toggle
    const hamburger: HTMLElement | null = document.querySelector('.hamburger');
    if (hamburger) {
        hamburger.addEventListener('click', toggleMobileMenu);
    }
    
    // Navigation link smooth scrolling
    const navLinks: NodeListOf<HTMLAnchorElement> = document.querySelectorAll('.nav-link[href^="#"]');
    navLinks.forEach((link: HTMLAnchorElement): void => {
        link.addEventListener('click', (e: Event): void => {
            e.preventDefault();
            const targetId: string = link.getAttribute('href')?.substring(1) || '';
            smoothScrollTo(targetId);
            
            // Close mobile menu if open
            const navMenu: HTMLElement | null = document.querySelector('.nav-menu');
            const hamburger: HTMLElement | null = document.querySelector('.hamburger');
            if (navMenu && hamburger && navMenu.classList.contains('active')) {
                navMenu.classList.remove('active');
                hamburger.classList.remove('active');
            }
        });
    });
    
    // Copy code buttons
    const copyButtons: NodeListOf<HTMLElement> = document.querySelectorAll('.copy-btn');
    copyButtons.forEach((button: HTMLElement): void => {
        button.addEventListener('click', (): void => copyCode(button));
    });
    
    // Start BitSacco chat animation when page loads
    setTimeout((): void => {
        // Start the continuous animation loop
        function startContinuousAnimation(): void {
            animateBitSaccoChat();
            // Repeat every 4 seconds
            setTimeout(startContinuousAnimation, 4000);
        }
        startContinuousAnimation();
    }, 2000);
    
    // Navbar scroll effect
    const navbar: HTMLElement | null = document.querySelector('.navbar');
    if (navbar) {
        window.addEventListener('scroll', (): void => {
            if (window.scrollY > 100) {
                navbar.classList.add('scrolled');
            } else {
                navbar.classList.remove('scrolled');
            }
        });
    }
    
    // Intersection Observer for animations
    const observerOptions: IntersectionObserverInit = {
        threshold: 0.1,
        rootMargin: '0px 0px -50px 0px'
    };
    
    const observer: IntersectionObserver = new IntersectionObserver((entries: IntersectionObserverEntry[]): void => {
        entries.forEach((entry: IntersectionObserverEntry): void => {
            if (entry.isIntersecting) {
                entry.target.classList.add('animate');
            }
        });
    }, observerOptions);
    
    // Observe elements for animation
    const animateElements: NodeListOf<HTMLElement> = document.querySelectorAll('.feature-card, .command-category');
    animateElements.forEach((element: HTMLElement): void => {
        observer.observe(element);
    });
});

// Export types and functions for potential module usage
export type { ChatMessage, BitcoinPrice, BitcoinPriceResponse };
export { fetchBitcoinPrice, animateBitSaccoChat, toggleMobileMenu, copyCode, smoothScrollTo };
