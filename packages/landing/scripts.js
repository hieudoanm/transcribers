// Copy buttons for install commands.
document.querySelectorAll('.code').forEach((block) => {
  const btn = block.querySelector('.copy-btn');
  const pre = block.querySelector('pre');
  if (!btn || !pre) return;

  btn.addEventListener('click', async () => {
    const text = pre.textContent.trim();
    try {
      await navigator.clipboard.writeText(text);
    } catch {
      // Fallback for non-secure contexts.
      const ta = document.createElement('textarea');
      ta.value = text;
      ta.style.position = 'fixed';
      ta.style.opacity = '0';
      document.body.appendChild(ta);
      ta.select();
      document.execCommand('copy');
      ta.remove();
    }
    btn.textContent = 'Copied';
    btn.classList.add('copied');
    setTimeout(() => {
      btn.textContent = 'Copy';
      btn.classList.remove('copied');
    }, 1500);
  });
});
