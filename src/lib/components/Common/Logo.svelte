<script lang="ts">
  let { size = 16, hasBackground = false } = $props<{ size?: number; hasBackground?: boolean }>();
</script>

<div class="logo-container" class:with-bg={hasBackground} style="--size: {size}px">
  <svg width="100%" height="100%" viewBox="0 0 100 100" fill="none" xmlns="http://www.w3.org/2000/svg">
    <!-- Premium Elegant Bolt -->
    <defs>
      <linearGradient id="bolt-grad-main" x1="20" y1="20" x2="80" y2="80" gradientUnits="userSpaceOnUse">
        <stop offset="0%" stop-color="#38BDF8"/> <!-- Sky Blue -->
        <stop offset="50%" stop-color="#818CF8"/> <!-- Indigo -->
        <stop offset="100%" stop-color="#C084FC"/> <!-- Purple -->
      </linearGradient>
      <filter id="premium-glow" x="-20%" y="-20%" width="140%" height="140%">
        <feGaussianBlur stdDeviation="3" result="blur"/>
        <feComposite in="SourceGraphic" in2="blur" operator="over"/>
      </filter>
    </defs>
    
    <!-- Outer accent layer -->
    <path 
      d="M55 10L25 55H45L40 90L75 45H55L60 10Z" 
      fill="white" 
      opacity="0.15"
      transform="translate(2, 2)"
    />
    
    <!-- Main Bolt body -->
    <path 
      d="M55 10L25 55H45L40 90L75 45H55L60 10Z" 
      fill="url(#bolt-grad-main)" 
      filter="url(#premium-glow)"
    />
    
    <!-- Inner highlights for depth -->
    <path 
      d="M56 12L28 53H47L43 85L72 43H53L56 12Z" 
      stroke="white" 
      stroke-width="0.5" 
      opacity="0.3"
    />
  </svg>
</div>

<style>
  .logo-container {
    width: var(--size);
    height: var(--size);
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }
  .logo-container.with-bg {
    background: linear-gradient(145deg, #0f172a, #020617);
    border: 1px solid rgba(255,255,255,0.1);
    border-radius: 6px;
    padding: 3px;
    box-shadow: 0 4px 12px rgba(0,0,0,0.4);
  }
</style>
