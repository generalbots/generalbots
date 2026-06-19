/* General Bots OS v15 — Three.js 3D Desktop Engine */
if (typeof window.Desktop3D === "undefined") {
  (function () {
    "use strict";

    class Desktop3D {
      constructor() {
        this.scene = null;
        this.camera = null;
        this.renderer = null;
        this.windows3D = {};
        this.initialized = false;
        this.bgCanvas = null;
        this.animationId = null;
        this.particleSystem = null;
        this.flipGroup = null;
        this.flipTargets = [];
      }

      init(containerEl) {
        if (this.initialized) return;
        if (typeof THREE === "undefined") {
          console.warn("Three.js not loaded — 3D effects disabled");
          return;
        }

        const rect = containerEl.getBoundingClientRect();
        const w = rect.width || window.innerWidth;
        const h = rect.height || window.innerHeight;

        this.scene = new THREE.Scene();
        this.camera = new THREE.PerspectiveCamera(60, w / h, 0.1, 2000);
        this.camera.position.set(0, 0, 800);

        this.renderer = new THREE.WebGLRenderer({ alpha: true, antialias: true });
        this.renderer.setSize(w, h);
        this.renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
        this.renderer.setClearColor(0x000000, 0);

        this.bgCanvas = this.renderer.domElement;
        this.bgCanvas.style.position = "absolute";
        this.bgCanvas.style.top = "0";
        this.bgCanvas.style.left = "0";
        this.bgCanvas.style.width = "100%";
        this.bgCanvas.style.height = "100%";
        this.bgCanvas.style.pointerEvents = "none";
        this.bgCanvas.style.zIndex = "0";
        containerEl.style.position = "relative";
        containerEl.appendChild(this.bgCanvas);

        this.createParticleField();
        this.createLights();

        this.flipGroup = new THREE.Group();
        this.scene.add(this.flipGroup);

        window.addEventListener("resize", () => this.onResize(containerEl));
        this.animate();
        this.initialized = true;
        console.log("Desktop3D initialized");
      }

      createParticleField() {
        const geom = new THREE.BufferGeometry();
        const count = 120;
        const positions = new Float32Array(count * 3);
        const colors = new Float32Array(count * 3);

        for (let i = 0; i < count; i++) {
          positions[i * 3] = (Math.random() - 0.5) * 1200;
          positions[i * 3 + 1] = (Math.random() - 0.5) * 800;
          positions[i * 3 + 2] = (Math.random() - 0.5) * 600 - 200;
          colors[i * 3] = 0.2 + Math.random() * 0.3;
          colors[i * 3 + 1] = 0.7 + Math.random() * 0.3;
          colors[i * 3 + 2] = 1.0;
        }

        geom.setAttribute("position", new THREE.BufferAttribute(positions, 3));
        geom.setAttribute("color", new THREE.BufferAttribute(colors, 3));

        const mat = new THREE.PointsMaterial({
          size: 2.5,
          vertexColors: true,
          blending: THREE.AdditiveBlending,
          depthWrite: false,
          transparent: true,
          opacity: 0.4,
        });

        this.particleSystem = new THREE.Points(geom, mat);
        this.scene.add(this.particleSystem);
      }

      createLights() {
        const ambient = new THREE.AmbientLight(0x222244, 0.5);
        this.scene.add(ambient);
        const dir = new THREE.DirectionalLight(0x6688cc, 0.6);
        dir.position.set(200, 300, 400);
        this.scene.add(dir);
      }

      onResize(containerEl) {
        if (!this.camera || !this.renderer) return;
        const rect = containerEl.getBoundingClientRect();
        const w = rect.width || window.innerWidth;
        const h = rect.height || window.innerHeight;
        this.camera.aspect = w / h;
        this.camera.updateProjectionMatrix();
        this.renderer.setSize(w, h);
      }

      animate() {
        if (!this.renderer) return;
        this.animationId = requestAnimationFrame(() => this.animate());

        if (this.particleSystem) {
          this.particleSystem.rotation.y += 0.00015;
          this.particleSystem.rotation.x += 0.00008;
        }

        if (this.flipGroup && this.flipGroup.children.length > 0) {
          const elapsed = performance.now() * 0.001;
          this.flipGroup.rotation.y += (Math.sin(elapsed * 0.8) * 0.02 - this.flipGroup.rotation.y) * 0.05;
        }

        this.renderer.render(this.scene, this.camera);
      }

      createWindowPlane(id, htmlContent) {
        if (!this.scene) return null;
        const w = 640;
        const h = 480;
        const geom = new THREE.PlaneGeometry(w, h, 4, 4);

        const canvas = document.createElement("canvas");
        canvas.width = 1024;
        canvas.height = 768;
        const ctx = canvas.getContext("2d");

        ctx.fillStyle = "rgba(20,22,35,0.92)";
        this.roundRect(ctx, 0, 0, canvas.width, canvas.height, 24);
        ctx.fill();
        ctx.fillStyle = "rgba(255,255,255,0.06)";
        this.roundRect(ctx, 0, 0, canvas.width, 48, 24);
        ctx.fill();
        ctx.fillStyle = "#88ccff";
        ctx.font = "bold 20px Inter, sans-serif";
        ctx.fillText(htmlContent || "Window", 24, 32);
        ctx.fillStyle = "#ff6b6b";
        ctx.beginPath();
        ctx.arc(canvas.width - 32, 24, 8, 0, Math.PI * 2);
        ctx.fill();
        ctx.fillStyle = "#ffd93d";
        ctx.beginPath();
        ctx.arc(canvas.width - 60, 24, 8, 0, Math.PI * 2);
        ctx.fill();
        ctx.fillStyle = "#6bcb77";
        ctx.beginPath();
        ctx.arc(canvas.width - 88, 24, 8, 0, Math.PI * 2);
        ctx.fill();

        const texture = new THREE.CanvasTexture(canvas);
        const mat = new THREE.MeshStandardMaterial({
          map: texture,
          side: THREE.DoubleSide,
          roughness: 0.4,
          metalness: 0.1,
          transparent: true,
          opacity: 0.95,
        });

        const mesh = new THREE.Mesh(geom, mat);
        mesh.userData = { id, width: w, height: h };
        this.flipGroup.add(mesh);
        this.flipTargets.push(mesh);
        this.windows3D[id] = mesh;
        return mesh;
      }

      roundRect(ctx, x, y, w, h, r) {
        ctx.beginPath();
        ctx.moveTo(x + r, y);
        ctx.lineTo(x + w - r, y);
        ctx.quadraticCurveTo(x + w, y, x + w, y + r);
        ctx.lineTo(x + w, y + h - r);
        ctx.quadraticCurveTo(x + w, y + h, x + w - r, y + h);
        ctx.lineTo(x + r, y + h);
        ctx.quadraticCurveTo(x, y + h, x, y + h - r);
        ctx.lineTo(x, y + r);
        ctx.quadraticCurveTo(x, y, x + r, y);
        ctx.closePath();
      }

      flipToWindow(id, duration = 600) {
        if (!this.flipGroup || !this.windows3D[id]) return;
        const mesh = this.windows3D[id];

        this.flipGroup.children.forEach((child) => {
          if (child !== mesh) child.visible = false;
        });
        mesh.visible = true;
        mesh.rotation.set(0, -Math.PI * 0.25, 0);
        mesh.position.set(0, 0, 0);
        mesh.scale.set(0.9, 0.9, 0.9);

        const start = performance.now();
        const startRot = mesh.rotation.y;
        const startScale = mesh.scale.x;

        const animateFlip = (now) => {
          const elapsed = now - start;
          const t = Math.min(elapsed / duration, 1.0);
          const ease = 1 - Math.pow(1 - t, 3);

          mesh.rotation.y = startRot * (1 - ease);
          mesh.scale.set(
            startScale + (1 - startScale) * ease,
            startScale + (1 - startScale) * ease,
            startScale + (1 - startScale) * ease
          );

          if (t < 1) {
            requestAnimationFrame(animateFlip);
          }
        };

        requestAnimationFrame(animateFlip);
      }

      removeWindow(id) {
        if (!this.windows3D[id]) return;
        const mesh = this.windows3D[id];
        this.flipGroup.remove(mesh);
        mesh.material.map?.dispose();
        mesh.material.dispose();
        mesh.geometry.dispose();
        delete this.windows3D[id];
        this.flipTargets = this.flipTargets.filter((m) => m !== mesh);
      }

      dispose() {
        if (this.animationId) cancelAnimationFrame(this.animationId);
        Object.keys(this.windows3D).forEach((id) => this.removeWindow(id));
        if (this.particleSystem) {
          this.particleSystem.geometry.dispose();
          this.particleSystem.material.dispose();
          this.scene.remove(this.particleSystem);
        }
        if (this.bgCanvas && this.bgCanvas.parentNode) {
          this.bgCanvas.parentNode.removeChild(this.bgCanvas);
        }
        if (this.renderer) {
          this.renderer.dispose();
          this.renderer = null;
        }
        this.scene = null;
        this.camera = null;
        this.initialized = false;
      }
    }

    window.Desktop3D = new Desktop3D();
  })();
}