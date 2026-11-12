size => new Promise(resolve => {
  if (location.origin !== 'https://web.whatsapp.com') {
    resolve(null);
    return;
  }

  function finish(result) {
    clearTimeout(timer);
    window.removeEventListener('click', pick, true);
    resolve(result);
  }

  async function pick(event) {
    if (!event.isTrusted || event.button !== 0) return;
    if (performance.now() >= deadline) {
      finish('timeout');
      return;
    }
    const { clientX: x, clientY: y } = event;
    const candidates = document.elementsFromPoint(x, y);
    const button = event.target.closest?.('button, [role="button"]');
    if (button) candidates.push(...button.querySelectorAll('img'));
    const image = candidates.find(element => {
      if (!(element instanceof HTMLImageElement) || !element.naturalWidth ||
          element.closest('#pane-side, [data-pre-plain-text], [data-ref]')) return false;
      const { left, top, right, bottom } = element.getBoundingClientRect();
      return x >= left && x < right && y >= top && y < bottom;
    });
    if (!image) return;
    event.preventDefault();
    event.stopImmediatePropagation();
    window.removeEventListener('click', pick, true);

    const canvas = document.createElement('canvas');
    canvas.width = canvas.height = size;
    const context = canvas.getContext('2d');

    function pixels(source) {
      // Resizing the canvas clears both its previous pixels and its origin-clean state.
      canvas.width = size;
      const side = Math.min(source.naturalWidth, source.naturalHeight);
      context.drawImage(source, (source.naturalWidth - side) / 2,
        (source.naturalHeight - side) / 2, side, side, 0, 0, size, size);
      const data = context.getImageData(0, 0, size, size).data;
      return Array.from(data, value => value.toString(16).padStart(2, '0')).join('');
    }

    try {
      finish(pixels(image));
    } catch {
      try {
        const copy = new Image();
        copy.crossOrigin = 'anonymous';
        copy.src = image.currentSrc;
        await copy.decode();
        finish(pixels(copy));
      } catch {
        finish('unreadable');
      }
    }
  }

  const deadline = performance.now() + 25000;
  const timer = setTimeout(() => finish('timeout'), 25000);
  window.addEventListener('click', pick, true);
})
