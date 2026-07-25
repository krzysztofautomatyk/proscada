<script lang="ts">
  import QRCode from "qrcode";
  import type { WidgetRendererProps } from "../../shared/types";
  import { configOf, readNumber, readString, tagNumber } from "../../shared/config";

  let { widget, tag = null }: WidgetRendererProps = $props();
  const config = $derived(configOf(widget));
  const configuredText = $derived(readString(config, "text", "ProSCADA"));
  const includeTagValue = $derived(readString(config, "source", "static") === "tag");
  const payload = $derived(includeTagValue && tag ? String(tagNumber(tag)) : configuredText);
  const errorCorrection = $derived(readString(config, "errorCorrection", "M") as "L" | "M" | "Q" | "H");
  const margin = $derived(Math.round(readNumber(config, "margin", 2, 0, 8)));
  let dataUrl = $state("");
  let error = $state("");

  $effect(() => {
    const value = payload;
    const level = errorCorrection;
    const quietZone = margin;
    let cancelled = false;
    dataUrl = "";
    error = "";
    if (!value) {
      error = "QR payload is empty";
      return;
    }
    void QRCode.toDataURL(value, {
      errorCorrectionLevel: level,
      margin: quietZone,
      width: 512,
      color: { dark: "#0f172a", light: "#ffffff" },
    })
      .then((url) => {
        if (!cancelled) dataUrl = url;
      })
      .catch((reason: unknown) => {
        if (!cancelled) {
          error = reason instanceof Error ? reason.message : "QR generation failed";
        }
      });
    return () => {
      cancelled = true;
    };
  });
</script>

<figure>
  {#if error}
    <div class="error" role="alert">▲ {error}</div>
  {:else if dataUrl}
    <img src={dataUrl} alt="QR code for {payload}" />
  {:else}
    <div class="loading">Generating QR…</div>
  {/if}
  <figcaption>{readString(config, "caption", "QR CODE")}</figcaption>
</figure>

<style>
  figure {
    width: 100%;
    height: 100%;
    box-sizing: border-box;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 4px;
    margin: 0;
    overflow: hidden;
    border: 1px solid #d8dee8;
    border-radius: 7px;
    background: #fff;
    padding: 5px;
  }
  img {
    min-height: 0;
    max-width: 100%;
    max-height: calc(100% - 16px);
    object-fit: contain;
    image-rendering: pixelated;
  }
  figcaption {
    color: #475569;
    font-size: 8px;
    font-weight: 800;
    letter-spacing: 0.06em;
  }
  .error,
  .loading {
    display: grid;
    min-height: 0;
    flex: 1;
    place-items: center;
    color: #64748b;
    font-size: 9px;
    text-align: center;
  }
  .error {
    color: #991b1b;
  }
</style>

