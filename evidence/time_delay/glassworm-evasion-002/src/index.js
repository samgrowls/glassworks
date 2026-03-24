/**
 * System Resource Detection
 * Checks CPU and memory to detect sandbox environments
 */

const os = require('os');

// Thresholds for sandbox detection
const MIN_CPU_COUNT = 2;
const MIN_MEMORY_GB = 2;
const MIN_UPTIME_SECONDS = 60;

/**
 * Check CPU count
 * Sandboxes often have only 1 CPU core
 * @returns {Object} - CPU check results
 */
function checkCPU() {
    const cpus = os.cpus();
    const cpuCount = cpus.length;
    
    const result = {
        count: cpuCount,
        model: cpus[0]?.model || 'unknown',
        speed: cpus[0]?.speed || 0,
        isSuspicious: cpuCount < MIN_CPU_COUNT
    };
    
    if (result.isSuspicious) {
        console.log(`[SUSPICIOUS] Low CPU count: ${cpuCount} (minimum: ${MIN_CPU_COUNT})`);
    }
    
    return result;
}

/**
 * Check total memory
 * Sandboxes often have limited RAM (<2GB)
 * @returns {Object} - Memory check results
 */
function checkMemory() {
    const totalMem = os.totalmem();
    const freeMem = os.freemem();
    const totalMemGB = totalMem / (1024 * 1024 * 1024);
    const freeMemGB = freeMem / (1024 * 1024 * 1024);
    
    const result = {
        totalGB: totalMemGB.toFixed(2),
        freeGB: freeMemGB.toFixed(2),
        usagePercent: ((1 - freeMem / totalMem) * 100).toFixed(2),
        isSuspicious: totalMemGB < MIN_MEMORY_GB
    };
    
    if (result.isSuspicious) {
        console.log(`[SUSPICIOUS] Low memory: ${result.totalGB}GB (minimum: ${MIN_MEMORY_GB}GB)`);
    }
    
    return result;
}

/**
 * Check system uptime
 * Fresh VMs/sandboxes have very low uptime
 * @returns {Object} - Uptime check results
 */
function checkUptime() {
    const uptime = os.uptime();
    const uptimeMinutes = Math.floor(uptime / 60);
    
    const result = {
        seconds: uptime,
        minutes: uptimeMinutes,
        hours: (uptime / 3600).toFixed(2),
        isSuspicious: uptime < MIN_UPTIME_SECONDS
    };
    
    if (result.isSuspicious) {
        console.log(`[SUSPICIOUS] Low uptime: ${uptimeMinutes} minutes (minimum: ${MIN_UPTIME_SECONDS} seconds)`);
    }
    
    return result;
}

/**
 * Combined system resource check
 * @returns {Object} - Combined results
 */
function checkSystemResources() {
    const cpu = checkCPU();
    const memory = checkMemory();
    const uptime = checkUptime();
    
    const suspiciousCount = [cpu.isSuspicious, memory.isSuspicious, uptime.isSuspicious]
        .filter(Boolean).length;
    
    return {
        cpu,
        memory,
        uptime,
        suspiciousCount,
        isLikelySandbox: suspiciousCount >= 2,
        shouldExecute: suspiciousCount < 2
    };
}

/**
 * Execute payload with resource-based delay
 * @param {Function} payload - Function to execute
 */
function executeWithResourceCheck(payload) {
    const resources = checkSystemResources();
    
    if (resources.isLikelySandbox) {
        console.log('[EVASION] Sandbox detected, delaying execution...');
        console.log('Suspicious indicators:', resources.suspiciousCount);
        
        // Long delay in sandbox (analyst will give up)
        setTimeout(() => {
            console.log('[EVASION] Still in sandbox, exiting silently');
            process.exit(0);
        }, 120000); // 2 minute delay
        
        return false;
    }
    
    console.log('[OK] System resources normal, executing payload');
    
    // Short delay in real environment
    setTimeout(() => {
        payload();
    }, 5000);
    
    return true;
}

// Export functions
module.exports = {
    checkCPU,
    checkMemory,
    checkUptime,
    checkSystemResources,
    executeWithResourceCheck,
    MIN_CPU_COUNT,
    MIN_MEMORY_GB,
    MIN_UPTIME_SECONDS
};

// Auto-execute check
const resources = checkSystemResources();
console.log('System resource check complete:', {
    isLikelySandbox: resources.isLikelySandbox,
    shouldExecute: resources.shouldExecute
});

// Execute payload if not in sandbox
if (resources.shouldExecute) {
    executeWithResourceCheck(() => {
        console.log('Payload executed successfully');
    });
}
