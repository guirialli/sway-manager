import os

# Ativa a flag de modo de teste para isolar arquivos de configuração reais do usuário
os.environ["SWAY_MANAGER_TEST_MODE"] = "1"
