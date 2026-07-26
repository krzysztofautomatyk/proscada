<script lang="ts">
  import AddVariableModalImpl from "../variables/AddVariableModal.svelte";

  interface Props {
    open: boolean;
    onClose: () => void;
  }

  let { open, onClose }: Props = $props();
</script>

<AddVariableModalImpl {open} {onClose} />
